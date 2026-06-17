# Script Layout

- `git/`
  - `get.cmd`
  - `get.ps1`
  - `commit.cmd`
  - `commit.ps1`
  - `.env`
  - `.env.example`
- `text/`
  - `normalize_text_files.py`
  - `check_line_endings.py`

## Common Commands

```powershell
scripts\git\get.cmd
scripts\git\commit.cmd
python scripts\text\normalize_text_files.py --root . --dry-run
python scripts\text\normalize_text_files.py --root . --fix
python scripts\text\check_line_endings.py --root .
```

## Notes

- `scripts\git\*.ps1` and `scripts\git\*.cmd` resolve the repository root from the current repo or `REPO_ROOT_OVERRIDE`.
- Git remotes are read from `scripts/git/.env` first, then from the repository root `.env` for compatibility.
- If `PREFERRED_REMOTE_URL` is not set, the git scripts reuse the current `origin` URL.
- `scripts\text\` utilities skip binary files and common generated directories.
