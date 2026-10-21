"""Installer contract tests: no network or normal user configuration."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


class InstallerTests(unittest.TestCase):
    def run_install(self, download_fail=False, install_fail=False, init_fail=False):
        with tempfile.TemporaryDirectory(prefix="steel-install-") as root:
            home = Path(root)
            tools = home / "tools"
            tools.mkdir()
            curl = tools / "curl"
            curl.write_text('''#!/bin/sh
if [ "$DOWNLOAD_FAIL" = 1 ]; then echo download-detail >&2; exit 22; fi
while [ $# -gt 0 ]; do
  if [ "$1" = -o ]; then cp "$FIXTURE" "$2"; exit 0; fi
  shift
done
exit 1
''')
            curl.chmod(0o755)
            fixture = home / "installer.sh"
            fixture.write_text('''#!/bin/sh
if [ "$INSTALL_FAIL" = 1 ]; then echo install-detail >&2; exit 1; fi
mkdir -p "$HOME/.steel/bin"
cat > "$HOME/.steel/bin/steel" <<'BIN'
#!/bin/sh
if [ "$1" = --version ]; then echo steel-cli-test; exit 0; fi
if [ "$1" = completion ]; then exit 1; fi
if [ "$1" = init ]; then
  echo "init source=$STEEL_ONBOARDING_FROM"
  if [ "$INIT_FAIL" = 1 ]; then echo auth-blocked >&2; exit 1; fi
fi
BIN
chmod +x "$HOME/.steel/bin/steel"
''')
            env = dict(os.environ, HOME=root, SHELL="", PATH=f"{tools}:/usr/bin:/bin",
                       FIXTURE=str(fixture), DOWNLOAD_FAIL=str(int(download_fail)),
                       INSTALL_FAIL=str(int(install_fail)), INIT_FAIL=str(int(init_fail)))
            result = subprocess.run(["sh", "install.sh", "--agent", "--from", "pi"],
                                    env=env, capture_output=True, text=True)
            # Simulate a separate agent tool command with unchanged parent PATH.
            second = None
            if not download_fail and not install_fail:
                second = subprocess.run(["sh", "-c", '"$HOME/.steel/bin/steel" --version'],
                                        env=env, capture_output=True, text=True)
            return result, second

    def test_clean_setup_and_subsequent_tool_reachability(self):
        result, second = self.run_install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("init source=pi", result.stdout)
        self.assertEqual(second.returncode, 0)
        self.assertIn("steel-cli-test", second.stdout)

    def test_download_failure_is_not_masked_by_pipeline(self):
        result, _ = self.run_install(download_fail=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("download-detail", result.stderr)
        self.assertIn("could not download", result.stderr)
        self.assertNotIn("Native CLI:", result.stdout)

    def test_install_failure_retains_stage_error(self):
        result, _ = self.run_install(install_fail=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("install-detail", result.stderr)
        self.assertIn("binary installation failed", result.stderr)

    def test_interrupted_setup_has_native_path_version_and_resume(self):
        result, second = self.run_install(init_fail=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Native CLI:", result.stdout)
        self.assertIn("steel-cli-test", result.stdout)
        self.assertIn('"$HOME/.steel/bin/steel" init --agent', result.stderr)
        self.assertEqual(second.returncode, 0)


if __name__ == "__main__":
    unittest.main()
