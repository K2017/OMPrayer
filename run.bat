set /p config="Config file: "
if defined config (
start /D "%cd%" target\release\prayer.exe "%config%"
)