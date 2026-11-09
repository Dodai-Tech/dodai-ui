#!/usr/bin/env nu
# ── Dodai UI · Create a Forgejo/Gitea release ───────────────────────────────
# Usage (in .forgejo/workflows/release.yml):
#   nu scripts/ci/forgejo_release.nu <tag> --api <server>/api/v1/repos/<owner>/<repo>
# Env: FORGEJO_TOKEN (the workflow passes secrets.GITHUB_TOKEN). Notes: RELEASE_NOTES.md.

export def release_payload [tag: string, notes: string] {
    { tag_name: $tag, name: $"Dodai UI ($tag)", body: $notes }
}

def main [tag: string, --api: string] {
    let headers = { Authorization: $"token ($env.FORGEJO_TOKEN)" }
    let notes = if ("RELEASE_NOTES.md" | path exists) { open --raw RELEASE_NOTES.md } else { "" }
    http post --content-type application/json --headers $headers $"($api)/releases" (release_payload $tag $notes) | ignore
    print $"🚀 release ($tag)"
}
