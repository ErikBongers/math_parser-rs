gcloud storage rsync www gs://mathparser/ --recursive --cache-control="private, max-age=0, no-transform" --exclude='pack[/\\]package\.json$"|"pack[/\\]\.gitignore$'
