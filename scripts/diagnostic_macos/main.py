import sys
import os
import sqlite3
from pathlib import Path
from pprint import pformat

def main():
    get_plugin_info()

def get_plugin_info():
    PLUG = "KClip"
    print("getting info for plugin " + PLUG)
    file_path = Path.home() / "Desktop" / "tempo-diagnostic.txt"

    file = open(file_path, "wt")

    dbs = get_plugin_dbs()

    file.write("found dbs:\n")
    file.write(pformat(dbs, indent=2, width=80) + "\n\n")

    for db in dbs:
        file.write("on db: " + str(db) + "\n")
        try:
            data = get_plug_in_db(db, PLUG)
            file.write("rows:\n")
            file.write(pformat(data, indent=2, width=80) + "\n")
        except Exception as e:
            file.write("error: " + e + "\n")
        file.write("\n")
    
    print("done. wrote info to " + str(file_path))

def get_plug_in_db(path, plug_name):
    conn = sqlite3.connect(path)
    cursor = conn.cursor()
    return [row for row in cursor.execute("SELECT * FROM plugins WHERE name LIKE ? || '%'", (plug_name,))]

def get_plugin_dbs():
    return [f for f in (Path.home() / "Library" /
                        "Application Support" / "Ableton" / "Live Database").glob(f'*.db')]

if __name__ == "__main__":
    if os.geteuid() != 0:
        print("failed to run main script as superuser")
    else:
        main()
