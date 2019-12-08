import shutil
import sys

version = sys.argv[1].replace(".", "")

shutil.move("./target/release/my_library.dll", "./artifacts/my_lib.cp{}-win_amd64.pyd".format(version))
