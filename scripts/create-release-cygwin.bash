set -euo pipefail

# Create directory of files to install
rm -Rf build/release/
mkdir -p build/release/

# Build manager app
(
    cd manager-app/
    cargo build --release --target x86_64-pc-windows-gnu
)
cp manager-app/target/x86_64-pc-windows-gnu/release/manager-app.exe build/release/mhb-util

# Build util config
(
    cd config-builder/
    npm install
    npm run build-config
)

# Build task switch alarm
(
    cd apps/task_switch_alarm/
    cargo build --release --target x86_64-pc-windows-gnu
)
cp apps/task_switch_alarm/target/x86_64-pc-windows-gnu/release/task_switch_alarm.exe build/release/task_switch_alarm
cp apps/task_switch_alarm/task_switch_alarm.bash build/release/
cp apps/task_switch_alarm/woomy.mp3 build/release/

# Copy echo script
cp manager-app/test/echo.bash build/release

# Copy install script
cp scripts/install-to-home.bash build/release

# Compress release
7z a build/mhb-release-cygwin.zip ./build/release/*