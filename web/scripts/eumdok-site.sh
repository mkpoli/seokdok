#!/bin/sh
# Make dist-eumdok, the eumdok.mkpo.li site, from a finished build: the 音讀 page as
# its front page, with the other product's pages and downloads left out.
set -eu
cd "$(dirname "$0")/.."
rm -rf dist-eumdok
cp -r dist dist-eumdok
cd dist-eumdok
rm -f index.html design.html privacy.html support.html dict.html sitemap.xml _redirects dict/browse.bin
mv eumdok.html index.html
rm -f downloads/seokdok-*
printf 'User-agent: *\nAllow: /\n\nSitemap: https://eumdok.mkpo.li/sitemap.xml\n' >robots.txt
printf '<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n  <url><loc>https://eumdok.mkpo.li/</loc></url>\n</urlset>\n' >sitemap.xml
