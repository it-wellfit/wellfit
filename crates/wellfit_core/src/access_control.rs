#![forbid(unsafe_code)]

//! Access Control - RBAC/ABAC Permission Model
//!
//! Implements Role-Based Access Control (RBAC) and Attribute-Based Access Control (ABAC)
//! for the Wellfit MTRIX system, aligned with ISO/IEC 27001:2022 Annex A controls:
//! - A.5.15 Access Control (rights allocation, least privilege)
//! - A.5.17 Authentication Information (MFA, identity verification)
//! - A.6.5 Segregation of Duties (SoD constraint checking)
//! - A.8.2 Privileged Access Management (privileged operation separation)
//! - A.8.16 Monitoring Behaviour (access decision audit logging)

use serde::{Deserialize, Serialize};

/// Permission level granted on a resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// Read-only access.
    Read,
    /// Write access (includes read).
    Write,
    /// Delete access (includes read and write).
    Delete,
    /// Full administrative access (includes read, write, delete).
    Admin,
}

impl PermissionLevel {
    /// Returns true if this level implies the required level (i.e. is equal or higher).
    pub fn implies(self, required: PermissionLevel) -> bool {
        let order = |l: PermissionLevel| match l {
            PermissionLevel::Read => 1,
            PermissionLevel::Write => 2,
            PermissionLevel::Delete => 3,
            PermissionLevel::Admin => 4,
        };
        order(self) >= order(required)
    }

    /// Lower-case label used in audit logs.
    pub fn label(self) -> &'static str {
        match self {
            PermissionLevel::Read => "read",
            PermissionLevel::Write => "write",
            PermissionLevel::Delete => "delete",
            PermissionLevel::Admin => "admin",
        }
    }
}

/// Resource classification for ABAC decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Classification {
    /// Publicly shareable information.
    Public,
    /// Internal organisational information.
    Internal,
    /// Confidential information requiring controlled access.
    Confidential,
    /// Strictly confidential information with need-to-know restriction.
    StrictlyConfidential,
}

impl Classification {
    /// Returns the numeric sensitivity order.
    pub fn level(self) -> u8 {
        match self {
            Classification::Public => 0,
            Classification::Internal => 1,
            Classification::Confidential => 2,
            Classification::StrictlyConfidential => 3,
        }
    }

    /// Lower-case label used in audit logs.
    pub fn label(self) -> &'static str {
        match self {
            Classification::Public => "public",
            Classification::Internal => "internal",
            Classification::Confidential => "confidential",
            Classification::StrictlyConfidential => "strictly_confidential",
        }
    }
}

/// A single permission grant: action + resource + classification scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission {
    /// Action the permission authorises (e.g. "mtrix:validate", "ledger:read").
    pub action: String,
    /// Resource pattern the permission applies to (e.g. "althci:*", "althvd:batch-20240601").
    pub resource: String,
    /// Maximum classification level this permission covers.
    pub classification: Classification,
    /// Minimum permission level required for this action.
    pub level: PermissionLevel,
}

impl Permission {
    /// Creates a new permission.
    pub fn new(
        action: impl Into<String>,
        resource: impl Into<String>,
        classification: Classification,
        level: PermissionLevel,
    ) -> Self {
        Permission {
            action: action.into(),
            resource: resource.into(),
            classification,
            level,
        }
    }

    /// Returns true if this permission covers the requested action/resource/classification/level.
    pub fn covers(
        &self,
        action: &str,
        resource: &str,
        classification: Classification,
        level: PermissionLevel,
    ) -> bool {
        let action_match = if self.action.ends_with(":*") {
            let prefix = &self.action[..self.action.len() - 2];
            action.starts_with(prefix)
        } else {
            self.action == action
        };
        let resource_match = if self.resource.ends_with(":*") {
            let prefix = &self.resource[..self.resource.len() - 2];
            resource.starts_with(prefix)
        } else {
            self.resource == resource
        };
        let class_ok = self.classification.level() >= classification.level();
        let level_ok = self.level.implies(level);
        action_match && resource_match && class_ok && level_ok
    }
}

/// Segregation of Duties constraint: two roles that must not be held by the same user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SodConstraint {
    /// First role identifier.
    pub role_a: String,
    /// Second role identifier.
    pub role_b: String,
    /// Human-readable reason for the constraint.
    pub reason: String,
}

/// A role with its assigned permissions and SoD constraints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    /// Unique role identifier.
    pub id: String,
    /// Human-readable role name.
    pub name: String,
    /// Permissions granted to this role.
    pub permissions: Vec<Permission>,
    /// SoD constraints this role participates in.
    pub sod_constraints: Vec<SodConstraint>,
}

impl Role {
    /// Creates a new role with the given id and name.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Role {
            id: id.into(),
            name: name.into(),
            permissions: Vec::new(),
            sod_constraints: Vec::new(),
        }
    }

    /// Adds a permission to this role.
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.push(permission);
    }

    /// Adds an SoD constraint to this role.
    pub fn add_sod_constraint(&mut self, constraint: SodConstraint) {
        self.sod_constraints.push(constraint);
    }
}

/// The result of an access check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessDecision {
    /// Access is granted.
    Allowed,
    /// Access is denied with the given reason.
    Denied { reason: String },
}

impl std::fmt::Display for AccessDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessDecision::Allowed => write!(f, "Allowed"),
            AccessDecision::Denied { reason } => write!(f, "Denied: {reason}"),
        }
    }
}

impl AccessDecision {
    /// Returns true if the decision is Allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, AccessDecision::Allowed)
    }
}

/// Context information for an access request (ABAC attributes).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessContext {
    /// Source IP address of the request.
    pub source_ip: Option<String>,
    /// Whether MFA was used for the current session.
    pub mfa_used: bool,
    /// Whether the request originates from a privileged session.
    pub privileged: bool,
    /// Time of the request (UTC RFC3339).
    pub timestamp: String,
    /// Additional custom attributes.
    pub attributes: serde_json::Value,
}

impl Default for AccessContext {
    fn default() -> Self {
        AccessContext {
            source_ip: None,
            mfa_used: false,
            privileged: false,
            timestamp: chrono::Utc::now().to_rfc3339(),
            attributes: serde_json::Value::Null,
        }
    }
}

/// An access request submitted for evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequest {
    /// Unique identifier for this request.
    pub request_id: String,
    /// User identifier making the request.
    pub user_id: String,
    /// Action being requested (e.g. "mtrix:validate").
    pub action: String,
    /// Resource being accessed (e.g. "althci:customer-995C").
    pub resource: String,
    /// Classification of the resource.
    pub classification: Classification,
    /// Permission level requested.
    pub level: PermissionLevel,
    /// ABAC context attributes.
    pub context: AccessContext,
}

impl AccessRequest {
    /// Creates a new access request with a generated UUID.
    pub fn new(
        user_id: impl Into<String>,
        action: impl Into<String>,
        resource: impl Into<String>,
        classification: Classification,
        level: PermissionLevel,
    ) -> Self {
        AccessRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            action: action.into(),
            resource: resource.into(),
            classification,
            level,
            context: AccessContext::default(),
        }
    }
}

/// An access decision with full audit context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRecord {
    /// Unique identifier for this access record.
    pub record_id: String,
    /// The original access request.
    pub request: AccessRequest,
    /// The decision rendered.
    pub decision: AccessDecision,
    /// Roles evaluated for this decision.
    pub roles_evaluated: Vec<String>,
    /// Whether any SoD constraint was violated.
    pub sod_violated: bool,
    /// Timestamp of the decision.
    pub decided_at: String,
}

impl AccessRecord {
    /// Creates a new access record from a request and decision.
    pub fn new(
        request: AccessRequest,
        decision: AccessDecision,
        roles_evaluated: Vec<String>,
        sod_violated: bool,
    ) -> Self {
        AccessRecord {
            record_id: uuid::Uuid::new_v4().to_string(),
            request,
            decision,
            roles_evaluated,
            sod_violated,
            decided_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Access controller implementing RBAC + ABAC with SoD constraint checking.
///
/// The controller evaluates access requests against assigned roles and
/// segregation-of-duties constraints. All decisions are auditable through
/// [`AccessController::record_access`].
pub struct AccessController {
    /// Roles keyed by role id.
    roles: std::sync::RwLock<std::collections::HashMap<String, Role>>,
    /// User-to-role assignments.
    user_roles: std::sync::RwLock<std::collections::HashMap<String, Vec<String>>>,
    /// Access audit log (in-memory, mirrored to persistent ledger via `record_access`).
    access_log: std::sync::RwLock<Vec<AccessRecord>>,
}

impl Default for AccessController {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessController {
    /// Creates a new empty access controller.
    pub fn new() -> Self {
        AccessController {
            roles: std::sync::RwLock::new(std::collections::HashMap::new()),
            user_roles: std::sync::RwLock::new(std::collections::HashMap::new()),
            access_log: std::sync::RwLock::new(Vec::new()),
        }
    }

    /// Registers a role in the controller.
    pub fn register_role(&self, role: Role) {
        self.roles.write().unwrap().insert(role.id.clone(), role);
    }

    /// Assigns a role to a user.
    pub fn assign_role(&self, user_id: &str, role_id: &str) {
        self.user_roles
            .write()
            .unwrap()
            .entry(user_id.to_string())
            .or_default()
            .push(role_id.to_string());
    }

    /// Removes a role assignment from a user.
    pub fn revoke_role(&self, user_id: &str, role_id: &str) {
        if let Some(roles) = self.user_roles.write().unwrap().get_mut(user_id) {
            roles.retain(|r| r != role_id);
        }
    }

    /// Returns the roles assigned to a user.
    pub fn user_roles(&self, user_id: &str) -> Vec<String> {
        self.user_roles
            .read()
            .unwrap()
            .get(user_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Evaluates an access request and returns a decision.
    ///
    /// The evaluation:
    /// 1. Collects all roles assigned to the user.
    /// 2. Checks each role's permissions for a match against the request.
    /// 3. Evaluates SoD constraints: if the user holds two roles in an SoD pair,
    ///    the decision is Denied.
    /// 4. Applies ABAC rules: MFA required for StrictlyConfidential resources;
    ///    privileged flag required for Admin-level operations.
    pub fn evaluate(&self, request: &AccessRequest) -> AccessDecision {
        let roles = self.user_roles(request.user_id.as_str());
        let mut roles_evaluated: Vec<String> = Vec::new();
        let mut sod_violated = false;
        let mut granted = false;

        // Check SoD constraints first: if user holds both roles in any SoD pair, deny.
        let role_set: std::collections::HashSet<&str> = roles.iter().map(|s| s.as_str()).collect();
        for role_id in &roles {
            if let Some(role) = self.roles.read().unwrap().get(role_id) {
                roles_evaluated.push(role.id.clone());
                for constraint in &role.sod_constraints {
                    if role_set.contains(&constraint.role_a.as_str())
                        && role_set.contains(&constraint.role_b.as_str())
                    {
                        sod_violated = true;
                    }
                }
            }
        }

        if sod_violated {
            return AccessDecision::Denied {
                reason: format!(
                    "segregation of duties violation: user {} holds mutually exclusive roles",
                    request.user_id
                ),
            };
        }

        // Check permissions: any role granting sufficient access allows the request.
        for role_id in &roles {
            if let Some(role) = self.roles.read().unwrap().get(role_id) {
                for permission in &role.permissions {
                    if permission.covers(
                        &request.action,
                        &request.resource,
                        request.classification,
                        request.level,
                    ) {
                        granted = true;
                        break;
                    }
                }
                if granted {
                    break;
                }
            }
        }

        if !granted {
            return AccessDecision::Denied {
                reason: format!(
                    "no role grants permission {} on {} for user {}",
                    request.action, request.resource, request.user_id
                ),
            };
        }

        // ABAC: MFA required for StrictlyConfidential resources.
        if request.classification == Classification::StrictlyConfidential
            && !request.context.mfa_used
        {
            return AccessDecision::Denied {
                reason: "MFA required for StrictlyConfidential resources".to_string(),
            };
        }

        // ABAC: privileged flag required for Admin-level operations.
        if request.level == PermissionLevel::Admin && !request.context.privileged {
            return AccessDecision::Denied {
                reason: "privileged session required for Admin-level operations".to_string(),
            };
        }

        AccessDecision::Allowed
    }

    /// Records an access decision in the in-memory audit log.
    pub fn record_access(
        &self,
        request: AccessRequest,
        decision: &AccessDecision,
        roles_evaluated: Vec<String>,
        sod_violated: bool,
    ) -> AccessRecord {
        let record = AccessRecord::new(request, decision.clone(), roles_evaluated, sod_violated);
        self.access_log.write().unwrap().push(record.clone());
        record
    }

    /// Returns all access records (audit log).
    pub fn access_log(&self) -> Vec<AccessRecord> {
        self.access_log.read().unwrap().clone()
    }

    /// Returns the number of access records.
    pub fn access_count(&self) -> usize {
        self.access_log.read().unwrap().len()
    }

    /// Returns the number of denied access records.
    pub fn denied_count(&self) -> usize {
        self.access_log
            .read()
            .unwrap()
            .iter()
            .filter(|r| !r.decision.is_allowed())
            .count()
    }

    /// Returns the roles currently registered.
    pub fn roles(&self) -> Vec<Role> {
        self.roles.read().unwrap().values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_level_implies() {
        assert!(PermissionLevel::Admin.implies(PermissionLevel::Read));
        assert!(PermissionLevel::Delete.implies(PermissionLevel::Write));
        assert!(PermissionLevel::Write.implies(PermissionLevel::Read));
        assert!(!PermissionLevel::Read.implies(PermissionLevel::Write));
    }

    #[test]
    fn test_permission_covers() {
        let perm = Permission::new(
            "mtrix:validate",
            "althci:*",
            Classification::Internal,
            PermissionLevel::Read,
        );
        assert!(perm.covers(
            "mtrix:validate",
            "althci:customer-1",
            Classification::Internal,
            PermissionLevel::Read
        ));
        assert!(!perm.covers(
            "mtrix:validate",
            "althci:customer-1",
            Classification::Confidential,
            PermissionLevel::Read
        ));
        assert!(!perm.covers(
            "mtrix:delete",
            "althci:customer-1",
            Classification::Internal,
            PermissionLevel::Read
        ));
        assert!(!perm.covers(
            "mtrix:validate",
            "althes:file-1",
            Classification::Internal,
            PermissionLevel::Read
        ));
    }

    #[test]
    fn test_access_controller_allow() {
        let controller = AccessController::new();
        let mut role = Role::new("analyst", "Data Analyst");
        role.add_permission(Permission::new(
            "mtrix:validate",
            "althci:*",
            Classification::Internal,
            PermissionLevel::Read,
        ));
        controller.register_role(role);
        controller.assign_role("user-1", "analyst");

        let request = AccessRequest::new(
            "user-1",
            "mtrix:validate",
            "althci:customer-995C",
            Classification::Internal,
            PermissionLevel::Read,
        );
        let decision = controller.evaluate(&request);
        assert!(decision.is_allowed());
    }

    #[test]
    fn test_access_controller_deny_no_permission() {
        let controller = AccessController::new();
        let mut role = Role::new("analyst", "Data Analyst");
        role.add_permission(Permission::new(
            "mtrix:validate",
            "althci:*",
            Classification::Internal,
            PermissionLevel::Read,
        ));
        controller.register_role(role);
        controller.assign_role("user-1", "analyst");

        let request = AccessRequest::new(
            "user-1",
            "mtrix:delete",
            "althci:customer-995C",
            Classification::Internal,
            PermissionLevel::Delete,
        );
        let decision = controller.evaluate(&request);
        assert!(!decision.is_allowed());
    }

    #[test]
    fn test_access_controller_sod_violation() {
        let controller = AccessController::new();
        let mut dev = Role::new("developer", "Developer");
        dev.add_sod_constraint(SodConstraint {
            role_a: "developer".to_string(),
            role_b: "deployer".to_string(),
            reason: "developer must not deploy to production".to_string(),
        });
        let mut deployer = Role::new("deployer", "Production Deployer");
        deployer.add_sod_constraint(SodConstraint {
            role_a: "developer".to_string(),
            role_b: "deployer".to_string(),
            reason: "developer must not deploy to production".to_string(),
        });
        controller.register_role(dev);
        controller.register_role(deployer);
        controller.assign_role("user-1", "developer");
        controller.assign_role("user-1", "deployer");

        let request = AccessRequest::new(
            "user-1",
            "mtrix:validate",
            "althci:customer-995C",
            Classification::Internal,
            PermissionLevel::Read,
        );
        let decision = controller.evaluate(&request);
        assert!(!decision.is_allowed());
        assert!(decision.to_string().contains("segregation of duties"));
    }

    #[test]
    fn test_access_controller_mfa_required() {
        let controller = AccessController::new();
        let mut role = Role::new("auditor", "Auditor");
        role.add_permission(Permission::new(
            "ledger:read",
            "althvd:*",
            Classification::StrictlyConfidential,
            PermissionLevel::Read,
        ));
        controller.register_role(role);
        controller.assign_role("user-1", "auditor");

        // Without MFA: denied.
        let request_no_mfa = AccessRequest::new(
            "user-1",
            "ledger:read",
            "althvd:batch-20240601",
            Classification::StrictlyConfidential,
            PermissionLevel::Read,
        );
        let decision_no_mfa = controller.evaluate(&request_no_mfa);
        assert!(!decision_no_mfa.is_allowed());

        // With MFA: allowed.
        let mut request_with_mfa = AccessRequest::new(
            "user-1",
            "ledger:read",
            "althvd:batch-20240601",
            Classification::StrictlyConfidential,
            PermissionLevel::Read,
        );
        request_with_mfa.context.mfa_used = true;
        let decision_with_mfa = controller.evaluate(&request_with_mfa);
        assert!(decision_with_mfa.is_allowed());
    }

    #[test]
    fn test_record_access_and_log() {
        let controller = AccessController::new();
        let mut role = Role::new("viewer", "Viewer");
        role.add_permission(Permission::new(
            "mtrix:read",
            "althci:*",
            Classification::Public,
            PermissionLevel::Read,
        ));
        controller.register_role(role);
        controller.assign_role("user-1", "viewer");

        let request = AccessRequest::new(
            "user-1",
            "mtrix:read",
            "althci:file-1",
            Classification::Public,
            PermissionLevel::Read,
        );
        let decision = controller.evaluate(&request);
        let roles = controller.user_roles("user-1");
        let record = controller.record_access(request, &decision, roles, false);

        assert_eq!(controller.access_count(), 1);
        assert_eq!(controller.denied_count(), 0);
        assert_eq!(record.decision, AccessDecision::Allowed);
        assert_eq!(record.request.user_id, "user-1");
    }
}
