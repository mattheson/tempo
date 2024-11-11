pyinstaller diagnostic.spec
codesign --force --deep --sign "-" dist/TempoDiagnostic.app
xattr -cr dist/TempoDiagnostic.app