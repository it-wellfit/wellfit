# FlapsErp Perf Baseline ??20261105-140417

## baseline: APP-NOT-RUNNING
- Launch FlapsErp then re-run: & "C:\SYSTEM_PROGRAM_DIRECTORY\FlapsErp\FlapsErp.exe"; or schedule via Task Scheduler 09:00/14:00 daily.
## com: OK
- Scripting.FileSystemObject.GetSpecialFolder(read-only)  n=1000  P50=0.2046ms P95=0.3816ms P99=0.5408ms mean=0.2684ms
## cdp: erpRunning=False
- port 9222: FOUND
