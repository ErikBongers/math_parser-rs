gsutil rsync -r www gs://mathparser/
gsutil setmeta -h "Cache-Control:private, max-age=0, no-transform" gs://mathparser/*.*