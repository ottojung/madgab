# Work item files

Create one Markdown file per standalone task in this directory using [../TEMPLATE.md](../TEMPLATE.md).

Files in this directory are discovered by scheduled orchestrators only when their YAML metadata contains `work_item: true`. Keep completed items in place with `state: done`; there is no need to move them to an archive directory.

Project continuation documents elsewhere under `docs/` may also be work items when explicitly marked with the same metadata.
