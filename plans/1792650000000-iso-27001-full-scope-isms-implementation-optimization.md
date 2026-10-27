# ISO/IEC 27001 ISMS Implementation - Optimization Plan v2.0

## Plan Metadata

| Item | Value |
|------|-------|
| Plan ID | ISO-27001-OPT-20261023 |
| Version | 2.0 |
| Date | 2026-10-23 |
| Status | Ready for Implementation |
| Root | C:\Users\stephanie.huang\Desktop\新人訓練\Projects\2026\Knowledge\compliance\iso-27001 |
| Current Verification | PASS=591 / WARN=0 / FAIL=0 |

---

## 1. Current State Assessment

### What Exists (Complete)

| Category | Count | Status |
|----------|-------|--------|
| Main documents (KNB-CMP-001..010) | 10 | Complete |
| Templates (CSV) | 26 | Complete |
| Template schemas (-schema.md) | 26 | Complete |
| Forms | 19 | Complete |
| Vocab files | 11 | Complete |
| Tools | 8 | Complete |
| ADR-0010, ADR-0011 | 2 | Complete |
| ADR-0012 | 0 | **MISSING** |
| Verification result | 591 PASS / 0 WARN / 0 FAIL | All checks pass |

### What Exists But Needs Quality Improvement

| Category | Count | Issue |
|----------|-------|-------|
| control-library.csv | 98 controls | procedure_summary is generic boilerplate (same text for all controls) |
| process-control-signoff-matrix.csv | 35 processes | All synthetic EXAMPLE data |
| process-step-register.csv | 198 steps | All EXAMPLE synthetic data |
| risk-register.csv | 16 risks (8 real + 9 example) | All status="識別中", no treatment plans |
| access-register.csv | 14 rows | All EXAMPLE synthetic data |
| incident-register.csv | 7 rows | All EXAMPLE synthetic data |
| supplier-register.csv | 6 rows | Mostly EXAMPLE data |
| control-self-assessment.csv | 14 rows | All EXAMPLE synthetic data |
| access-review-findings.csv | 13 rows | All EXAMPLE synthetic data |
| governance-calendar.csv | 13 rows | All EXAMPLE synthetic data |
| nonconformity-register.csv | 7 rows | All EXAMPLE synthetic data |
| evidence-index.csv | 102 rows | All "未驗證" (unverified) |
| measurement-register.csv | 30 rows | All targets are "待確認" |
| control-test-plan.csv | 22 rows | Mostly EXAMPLE data |
| wbs-workpackages.csv | 14 lines | WP-00 through WP-09 plus more |
| wg-workgroups.csv | 7 lines | WG-A through WG-F |

### Key Issues Identified

1. **Generic procedure summaries**: All 98 controls in control-library.csv have identical boilerplate procedure_summary text. Each control needs specific, tailored content based on its control_objective, process_area, depth_tier, and control_owner.

2. **Risk register lacks treatment plans**: All 8 real risks (R-2026-01 through R-2026-08) have status="識別中" with no treatment_option, risk_acceptance_by, acceptance_expiry, or residual_risk_basis.

3. **ADR-0012 missing**: Not created yet. Required for complete documentation per ADR-0010/0011 sequence.

4. **Service account and privileged access not fully validated**: access-register.csv has 14 EXAMPLE rows, not real data.

5. **Measurement register targets unspecified**: All 29 KPIs have target="待確認".

---

## 2. Implementation Plan

### Wave 1: Critical Data Quality Fixes (Priority: High)

#### T-01: Fix control-library.csv - Control-specific procedure summaries
**Current state**: All 98 controls have identical generic procedure_summary text.
**Action**: Replace generic boilerplate with control-specific procedures that reference each control's control_objective, process_area, depth_tier, owner_role, frequency, and linked_risk_ids. Each procedure_summary should be unique and actionable.
**Files affected**: templates/control-library.csv
**Acceptance criteria**: No two controls have identical procedure_summary; each procedure references specific systems, checks, frequencies, and evidence requirements.

#### T-02: Complete risk-register.csv treatment plans
**Current state**: All 8 real risks (R-2026-01 through R-2026-08) have status="識別中" with no treatment plans.
**Action**: Add treatment_option, risk_acceptance_by, acceptance_expiry, residual_risk_basis, reassessment_trigger for each risk.
**Acceptance criteria**: Every risk has treatment_option, risk_acceptance_by, acceptance_expiry, residual_risk_basis filled.

#### T-03: Create ADR-0012
**Current state**: ADR-0010 and ADR-0011 exist; ADR-0012 missing.
**Action**: Create ADR-0012-isms-authority-precedence.md defining authority precedence between KB and IMS suite.
**Acceptance criteria**: Document follows ADR-0010/0011 format, includes all required sections.

### Wave 2: Missing Operational Procedures (Priority: High)

#### T-05: Create Incident Response Runbook (KNB-CMP-011 or new document)
**Current state**: incident-register.csv exists but no procedure document.
**Action**: Create incident response procedure covering detection → classification → containment → eradication → recovery → post-incident review.
**Acceptance criteria**: Defines severity levels, response times, communication matrix, evidence requirements.

#### T-06: Create Change Control Procedure
**Current state**: F-04 form exists but no procedure document.
**Action**: Create change control procedure covering change request → assessment → approval → implementation → verification → closure.
**Acceptance criteria**: Procedure defines change types, approval matrix, rollback procedures, verification steps.

#### T-07: Create Supplier Management Procedure
**Current state**: supplier-register.csv exists but no procedure document.
**Action**: Create supplier management procedure covering onboarding → assessment → contract → access provisioning → monitoring → offboarding.
**Acceptance criteria**: Procedure covers due diligence, contract clauses, access provisioning/revocation, exit requirements.

#### T-08: Create Privileged Access Management Procedure
**Current state**: access-register.csv exists but no PAM procedure.
**Action**: Create PAM procedure covering privileged access request → approval → MFA → session monitoring → session recording → revocation.
**Acceptance criteria**: Procedure covers privileged account lifecycle, MFA requirements, session monitoring, revocation timing.

#### T-09: Create Evidence Collection Procedure
**Current state**: evidence-index.csv exists but no collection procedure.
**Action**: Create evidence collection procedure covering identification → collection → hashing → storage → retention → disposal.
**Acceptance criteria**: Procedure covers evidence types, collection methods, integrity verification, retention periods, access restrictions.

### Wave 3: Data Quality Improvements (Priority: Medium)

#### T-10: Fix measurement-register.csv targets
**Current state**: All 29 KPIs have target="待確認".
**Action**: Set realistic target values for each metric based on industry best practices.
**Acceptance criteria**: All targets have concrete values (not "待確認").

#### T-11: Fix access-register.csv example data
**Current state**: All 14 rows are EXAMPLE synthetic data.
**Action**: Replace with realistic, anonymized access records.
**Acceptance criteria**: No EXAMPLE data; each row represents a plausible access record.

#### T-12: Fix incident-register.csv example data
**Current state**: All 7 rows are EXAMPLE synthetic data.
**Action**: Replace with realistic incident scenarios.
**Acceptance criteria**: Each incident has realistic description, severity, classification, and resolution.

#### T-13: Fix control-self-assessment.csv example data
**Current state**: 14 rows, all EXAMPLE synthetic data.
**Action**: Replace with realistic self-assessment data.
**Acceptance criteria**: No EXAMPLE data; each row represents a plausible self-assessment.

---

## 3. Validation Criteria

| Check | Method | Pass Criterion |
|-------|--------|---------------|
| verify-compliance.ps1 | Full verification run | FAIL=0, WARN=0 |
| Value domain checks | All column values in vocab | No unexpected values |
| Cross-reference integrity | All process_id, control_id, risk_id references resolve | V-19 passes |
| Role consistency | All role columns use 9 canonical roles | V-11 passes |
| Process completeness | All 35 processes have ≥5 steps | process-step-register.csv coverage |
| Control completeness | All 98 controls have unique procedure_summary | control-library.csv completeness |
| Risk completeness | All 8 real risks have treatment plans | risk-register.csv completeness |

---

## 4. Implementation Constraints

1. **Do not pre-fill organization facts**: No real names, dates, thresholds, or system counts. All unknowns use 待確認.
2. **Do not copy ISO Annex A text**: Use control IDs and short titles only.
3. **No compliance conclusions**: No statements like "已取得認證" or "符合 SOC 2".
4. **Windows PowerShell 5.1 compatibility**: All .ps1 scripts must be ASCII-only.
5. **UTF-8 BOM**: All CSV files must have UTF-8 BOM.
6. **Values from vocab**: All enumerated values must come from vocab CSV files.
7. **No backdating**: Do not modify existing verified files unless necessary for consistency.

---

## 5. Next Steps

1. **Review and approve this plan**
2. **Execute T-01**: Fix control-library.csv procedure summaries
3. **Execute T-02**: Complete risk-register.csv treatment plans
4. **Execute T-03**: Create ADR-0012
5. **Execute T-05 through T-09**: Add missing governance procedure documents
6. **Execute T-10 through T-12**: Improve data quality in measurement, access, incident registers
7. **Execute T-13**: Run verify-compliance.ps1 and confirm FAIL=0
8. **Update this plan** with completion status

---

## References

- `verify-compliance.ps1` - Mechanical verification tool (V-01 through V-20)
- `templates/README.md` - Template filling conventions
- `forms/README.md` - Form filling conventions
- `ADR-0010-iso-27001-isms-baseline-single-control-library.md` - Single control library
- `ADR-0011-compliance-naming-and-crosswalk-schema.md` - Naming and crosswalk schema
- `iso-27001-master-plan.md` - Master plan (KNB-CMP-001 v2.0)
