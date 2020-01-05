import subprocess

p = subprocess.Popen(["cargo", "build", "--release"])
p.wait()

if p.returncode == 0:
    from pathlib import Path
    output_path = Path("target") / "release" / "my_library.dll"
    src_path = Path("src") / "my_library.cp37-win_amd64.pyd"

    import shutil
    shutil.move(output_path, src_path)


