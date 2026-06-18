# Script Layout

- `git/`
  - `get.cmd`
  - `get.ps1`
  - `commit.cmd`
  - `commit.ps1`
  - `.env`
  - `.env.example`
- `vercel/`
  - `dev_local.cmd`
  - `dev_local.ps1`
  - `link_project.cmd`
  - `link_project.ps1`
  - `.env.example`
- `text/`
  - `normalize_text_files.py`
  - `check_line_endings.py`

## Common Commands

```powershell
scripts\git\get.cmd
scripts\git\commit.cmd
scripts\vercel\dev_local.cmd
scripts\vercel\link_project.cmd
python scripts\text\normalize_text_files.py --root . --dry-run
python scripts\text\normalize_text_files.py --root . --fix
python scripts\text\check_line_endings.py --root .
```

## Notes

- `scripts\git\*.ps1` and `scripts\git\*.cmd` resolve the repository root from the current repo or `REPO_ROOT_OVERRIDE`.
- Git remotes are read from `scripts/git/.env` first, then from the repository root `.env` for compatibility.
- If `PREFERRED_REMOTE_URL` is not set, the git scripts reuse the current `origin` URL.
- `scripts\text\` utilities skip binary files and common generated directories.
- `scripts\vercel\` reads `scripts/vercel/.env` when present, otherwise falls back to `.env.example`.
