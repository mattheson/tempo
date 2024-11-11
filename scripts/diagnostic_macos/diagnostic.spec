# -*- mode: python ; coding: utf-8 -*-

block_cipher = None

a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=[],
    datas=[],
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False
)

b = Analysis(
    ['launch.py'],
    pathex=[],
    binaries=[],
    datas=[],
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False
)

pyz_a = PYZ(
    a.pure,
    a.zipped_data,
    cipher=block_cipher
)

pyz_b = PYZ(
    b.pure,
    b.zipped_data,
    cipher=block_cipher
)

exe_a = EXE(
    pyz_a,
    a.scripts,
    [],
    exclude_binaries=True,
    name='main',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)

# EXE for launcher script
exe_b = EXE(
    pyz_b,
    b.scripts,
    [],
    exclude_binaries=True,
    name='TempoDiagnostic',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)

coll = COLLECT(
    exe_a,
    exe_b,
    a.binaries,
    a.zipfiles,
    a.datas,
    b.binaries,
    b.zipfiles,
    b.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='TempoDiagnostic'
)

app = BUNDLE(
    coll,
    name='TempoDiagnostic.app',
    icon=None,
    bundle_identifier=None,
    info_plist={
        'LSBackgroundOnly': False,
        'NSHighResolutionCapable': True,
        'CFBundleShortVersionString': '1.0.0',
        'CFBundleVersion': '1.0.0',
        'NSRequiresAquaSystemAppearance': False,
        'CFBundleExecutable': 'TempoDiagnostic'
    }
)