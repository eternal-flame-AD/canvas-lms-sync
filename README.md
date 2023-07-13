# canvas-lms-sync

Synchronizes your course files and modules on Canvas LMS to your local machine.

## Usage

```bash
cargo install canvas-lms-sync

echo "
token: xxxxxx
host: "https://canvas.instructure.com/"
courseid: 123456
usemodules: true # whether to find files in "modules" or "files" section
" > canvas-sync.yml

canvas-sync
```