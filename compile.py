import subprocess
import os
import platform
import sys

process = subprocess.Popen(["cargo", "clean"])
process.wait()
process = subprocess.Popen(["cargo", "build", "--release"])
process.wait()

if process.returncode == 0:
    from pathlib import Path
    import shutil

    if platform.system() == "Linux":
        output_path = Path("target") / "release" / "libmy_library.so"
        target_path = Path("src") / "my_library.so"
        if target_path.exists():
            os.remove(target_path)
        shutil.copy(output_path, target_path)

    elif platform.system() == "Windows":
        major = sys.version_info.major
        minor = sys.version_info.minor
        output_path = Path("target") / "release" / "my_library.dll"
        target_path = Path("src") / f"my_library.cp{major}{minor}-win_amd64.pyd"
        if target_path.exists():
            os.remove(target_path)
        shutil.copy(output_path, target_path)

    # TODO MacOS
