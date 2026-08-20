#!/usr/bin/env python3
"""
Extract release notes and title from CHANGELOG.md for a given version or tag.
"""

import argparse
import os
import re
import sys


def parse_version(tag_or_version: str) -> str:
    """Extract clean version number (e.g. '1.9.5' from 'refs/tags/v1.9.5' or 'v1.9.5')."""
    clean = tag_or_version.split('/')[-1]
    if clean.startswith(('v', 'V')):
        clean = clean[1:]
    return clean.strip()


def extract_section(version: str, changelog_path: str = 'CHANGELOG.md'):
    """Extract release title and body markdown from CHANGELOG.md for the requested version."""
    clean_ver = parse_version(version)
    if not os.path.isfile(changelog_path):
        raise FileNotFoundError(f"Changelog file not found: {changelog_path}")

    with open(changelog_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Split on headers starting with ## [
    sections = re.split(r'(?m)(?=^##\s*\[)', content)
    for s in sections:
        lines = s.strip().splitlines()
        if not lines or not lines[0].startswith('##'):
            continue
        header = lines[0].strip()
        m = re.match(r'^##\s*\[([^\]]+)\](?:\s*[—–-]\s*(.*))?', header)
        if m:
            ver = m.group(1).strip()
            subtitle = m.group(2).strip() if m.group(2) else ''
            if ver == clean_ver:
                body = '\n'.join(lines[1:]).strip()
                title = f"v{ver} — {subtitle}" if subtitle else f"v{ver}"
                return {
                    'found': True,
                    'version': ver,
                    'title': title,
                    'subtitle': subtitle,
                    'body': body,
                }

    return {
        'found': False,
        'version': clean_ver,
        'title': f"v{clean_ver}",
        'subtitle': '',
        'body': '',
    }


def main():
    parser = argparse.ArgumentParser(description="Extract changelog notes for a version.")
    parser.add_argument("version", help="Version or tag to extract (e.g. v1.9.5, 1.9.5, refs/tags/v1.9.5)")
    parser.add_argument("--changelog", default="CHANGELOG.md", help="Path to CHANGELOG.md (default: CHANGELOG.md)")
    parser.add_argument("--output-notes", help="File path to write release notes markdown to")
    parser.add_argument("--output-title", help="File path to write release title to")
    parser.add_argument("--github-output", action="store_true", help="Append outputs to $GITHUB_OUTPUT if set")

    args = parser.parse_args()

    result = extract_section(args.version, args.changelog)

    if not result['found']:
        print(f"Warning: No changelog entry found for version '{args.version}' (parsed as '{result['version']}') in {args.changelog}", file=sys.stderr)

    if args.output_notes:
        with open(args.output_notes, 'w', encoding='utf-8') as f:
            f.write((result['body'] + '\n') if result['body'] else '')

    if args.output_title:
        with open(args.output_title, 'w', encoding='utf-8') as f:
            f.write(result['title'] + '\n')

    github_output_path = os.environ.get('GITHUB_OUTPUT')
    if args.github_output and github_output_path:
        with open(github_output_path, 'a', encoding='utf-8') as f:
            f.write(f"version={result['version']}\n")
            f.write(f"title={result['title']}\n")
            f.write(f"found={str(result['found']).lower()}\n")
            if args.output_notes:
                f.write(f"notes_file={args.output_notes}\n")

    # If neither output file is specified, print title and body to stdout
    if not args.output_notes and not args.output_title:
        print(f"=== Title: {result['title']} ===")
        print(result['body'])


if __name__ == '__main__':
    main()
