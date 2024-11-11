import os
import sys
import subprocess
import tempfile

def create_terminal_script():
    bundle_dir = os.path.dirname(os.path.abspath(sys.executable))
    main_script = os.path.join(bundle_dir, 'main')

    script_content = ""

    # thank you macos
    if "AppTranslocation" in str(bundle_dir):
        script_content = f'''
        #!/bin/bash
        clear
        echo "please move this application to any directory besides your downloads and run it again"
        read -p "press enter to exit..."
        osascript -e 'tell application "Terminal" to close front window' &
        '''

    else:
        script_content = f'''
        #!/bin/bash
        clear
        echo "tempo diagnostic tool"
        echo
        echo "right now this dumps info about KClip3 to your Desktop so i can debug, please send me the resulting file"
        echo
        echo "this tool needs admin to run, please enter your password:"
        
        sudo "{main_script}"

        read -p "press enter to exit..."
        osascript -e 'tell application "Terminal" to close front window' &
        '''

    with tempfile.NamedTemporaryFile(delete=False, suffix='.sh') as temp_script:
        temp_script.write(script_content.encode())
        script_path = temp_script.name

    os.chmod(script_path, 0o755)
    return script_path

def main():
    script_path = create_terminal_script()

    applescript = f'''
    tell application "Terminal"
        activate
        do script "{script_path}"
    end tell
    '''
    subprocess.run(['osascript', '-e', applescript])

if __name__ == '__main__':
    main()
