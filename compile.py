import subprocess
import os
import platform

# print(platform.system())

p = subprocess.Popen(["cargo", "build", "--release"])
p.wait()

if p.returncode == 0:
    from pathlib import Path
    import shutil

    if platform.system() == "Linux":
        output_path = Path("target") / "release" / "libmy_library.so"
        src_path = Path("src") / "my_library.so"
        if src_path.exists():
            os.remove(src_path)
        shutil.copy(output_path, src_path)

    else:
        output_path = Path("target") / "release" / "my_library.dll"
        src_path = Path("src") / "my_library.cp37-win_amd64.pyd"
        if src_path.exists():
            os.remove(src_path)
        shutil.copy(output_path, src_path)
