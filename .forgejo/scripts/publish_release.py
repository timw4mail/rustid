#!/usr/bin/env python3
"""
Publish or update a release on Forgejo and upload release assets.
"""

import argparse
import glob
import json
import os
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request


def make_request(url, method="GET", headers=None, data=None):
    if headers is None:
        headers = {}
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req) as resp:
            content = resp.read()
            return resp.status, content
    except urllib.error.HTTPError as e:
        content = e.read()
        return e.code, content


def main():
    parser = argparse.ArgumentParser(description="Publish release and assets to Forgejo.")
    parser.add_argument("--server-url", default=os.environ.get("FORGEJO_SERVER_URL") or os.environ.get("GITHUB_SERVER_URL", ""), help="Forgejo server URL")
    parser.add_argument("--repo", default=os.environ.get("FORGEJO_REPOSITORY") or os.environ.get("GITHUB_REPOSITORY", ""), help="Owner/repo slug")
    parser.add_argument("--token", default=os.environ.get("FORGEJO_TOKEN") or os.environ.get("GITHUB_TOKEN", ""), help="API Token")
    parser.add_argument("--tag", default=os.environ.get("FORGEJO_TAG") or os.environ.get("GITHUB_REF_NAME", ""), help="Release tag name")
    parser.add_argument("--title-file", default="release_title.txt", help="Path to release title text file")
    parser.add_argument("--notes-file", default="release_notes.md", help="Path to release notes markdown file")
    parser.add_argument("--assets", nargs="*", default=[], help="Asset files or globs to upload")

    args = parser.parse_args()

    server_url = args.server_url.rstrip("/")
    if not server_url:
        sys.exit("Error: --server-url, FORGEJO_SERVER_URL, or GITHUB_SERVER_URL is required")
    if not args.repo:
        sys.exit("Error: --repo, FORGEJO_REPOSITORY, or GITHUB_REPOSITORY is required")
    if not args.token:
        sys.exit("Error: --token, FORGEJO_TOKEN, or GITHUB_TOKEN is required")
    if not args.tag:
        sys.exit("Error: --tag, FORGEJO_TAG, or GITHUB_REF_NAME is required")

    title = args.tag
    if os.path.isfile(args.title_file):
        with open(args.title_file, "r", encoding="utf-8") as f:
            t = f.read().strip()
            if t:
                title = t

    notes = ""
    if os.path.isfile(args.notes_file):
        with open(args.notes_file, "r", encoding="utf-8") as f:
            notes = f.read().strip()

    headers = {
        "Authorization": f"token {args.token}",
        "Accept": "application/json",
    }

    api_base = f"{server_url}/api/v1/repos/{args.repo}"
    print(f"Connecting to Forgejo API: {api_base}")

    # Check if release exists
    tag_url = f"{api_base}/releases/tags/{urllib.parse.quote(args.tag)}"
    status, body = make_request(tag_url, method="GET", headers=headers)

    release_id = None
    if status == 200:
        release_data = json.loads(body.decode("utf-8"))
        release_id = release_data["id"]
        print(f"Found existing release ID {release_id} for tag {args.tag}, updating metadata...")
        patch_url = f"{api_base}/releases/{release_id}"
        patch_body = json.dumps({"name": title, "body": notes}).encode("utf-8")
        patch_headers = dict(headers)
        patch_headers["Content-Type"] = "application/json"
        p_status, p_body = make_request(patch_url, method="PATCH", headers=patch_headers, data=patch_body)
        if p_status not in (200, 201):
            sys.exit(f"Failed to update release: {p_status} {p_body.decode('utf-8', errors='replace')}")
    elif status == 404:
        print(f"Release for tag {args.tag} does not exist. Creating new release...")
        create_url = f"{api_base}/releases"
        create_data = {
            "tag_name": args.tag,
            "name": title,
            "body": notes,
            "draft": False,
            "prerelease": False,
        }
        commitish = os.environ.get("GITHUB_SHA")
        if commitish:
            create_data["target_commitish"] = commitish
        create_payload = json.dumps(create_data).encode("utf-8")
        create_headers = dict(headers)
        create_headers["Content-Type"] = "application/json"
        c_status, c_body = make_request(create_url, method="POST", headers=create_headers, data=create_payload)
        if c_status not in (200, 201):
            sys.exit(f"Failed to create release: {c_status} {c_body.decode('utf-8', errors='replace')}")
        release_data = json.loads(c_body.decode("utf-8"))
        release_id = release_data["id"]
        print(f"Created release ID {release_id} for tag {args.tag}")
    else:
        sys.exit(f"Unexpected response when checking release: {status} {body.decode('utf-8', errors='replace')}")

    # Expand asset globs
    asset_files = []
    for pattern in args.assets:
        matches = glob.glob(pattern)
        if not matches:
            if os.path.exists(pattern):
                asset_files.append(pattern)
            else:
                print(f"Warning: Asset pattern '{pattern}' matched no files.")
        else:
            asset_files.extend(matches)

    if not asset_files:
        print("No assets specified or found to upload.")
        return

    # Check existing attachments to delete duplicates (clobber)
    assets_url = f"{api_base}/releases/{release_id}/assets"
    a_status, a_body = make_request(assets_url, method="GET", headers=headers)
    existing_assets = {}
    if a_status == 200:
        for item in json.loads(a_body.decode("utf-8")):
            existing_assets[item["name"]] = item["id"]

    for file_path in asset_files:
        if not os.path.isfile(file_path):
            continue
        file_name = os.path.basename(file_path)
        if file_name in existing_assets:
            print(f"Deleting existing asset '{file_name}' (ID: {existing_assets[file_name]})...")
            del_url = f"{api_base}/releases/{release_id}/assets/{existing_assets[file_name]}"
            d_status, _ = make_request(del_url, method="DELETE", headers=headers)
            if d_status not in (200, 204):
                print(f"Warning: failed to delete old asset {file_name}: HTTP {d_status}")

        print(f"Uploading asset '{file_name}' ({os.path.getsize(file_path)} bytes)...")
        upload_url = f"{api_base}/releases/{release_id}/assets?name={urllib.parse.quote(file_name)}"
        
        curl_cmd = [
            "curl", "-sf", "-X", "POST",
            "-H", f"Authorization: token {args.token}",
            "-F", f"attachment=@{file_path}",
            upload_url
        ]
        res = subprocess.run(curl_cmd, capture_output=True, text=True)
        if res.returncode != 0:
            sys.exit(f"Failed to upload asset {file_name}: {res.stderr}")
        print(f"Successfully uploaded '{file_name}'")

    print(f"Release {args.tag} published successfully!")


if __name__ == "__main__":
    main()
