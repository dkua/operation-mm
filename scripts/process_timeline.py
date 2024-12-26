import csv
import html
import json
import requests

from argparse import ArgumentParser
from hashlib import md5
from mimetypes import guess_extension
from pathlib import Path

def process(args, line):
    event = dict()
    event["date"] = line["Date"]
    event["type"] = line["EventType"]
    event["title"] = line["Event"]
    event["subtitle"] = line["Subtitle"]

    event["collabed_with"] = [c.strip() for c in line["Collab Members"].split(",") if c]

    event["media"] = process_media(
        args,
        line["Link"],
        line["Image"],
    )
    return event

def process_media(args, link, image):
    media = dict()
    media["is_youtube"] = True
    url = requests.utils.urlparse(link)
    params = dict(p.split('=') for p in html.unescape(url.query).split('&') if url.query)

    if url.netloc == "youtu.be":
        media["video_id"] = url.path.strip("/")
    elif url.netloc == "www.youtube.com" and url.path.startswith("/watch/"):
        media["video_id"] = url.path.replace("/watch/", "")
    elif url.netloc == "www.youtube.com" and url.path.startswith("/live/"):
        media["video_id"] = url.path.replace("/live/", "")
    elif url.netloc == "www.youtube.com" and url.path.startswith("/watch"):
        media["video_id"] = params["v"]
    else:
        media["is_youtube"] = False
    
    if media["is_youtube"]:
        media["path"] = f"https://i.ytimg.com/vi/{media['video_id']}/maxresdefault.jpg"
        if "t" in params:
            media["video_start"] = params["t"].split("s")[0]
    else:
        media["link"] = link
    
    if image:
        media["path"] = download_image(args, image)
    return media

def download_image(args, image_url):
    resp = requests.get(image_url)
    if resp.status_code == 200:
        ext = guess_extension(resp.headers['Content-Type'].partition(';')[0].strip())
        hash_id = md5(image_url.encode("utf-8")).hexdigest()
        filepath = f"{args.image_path}/{hash_id}{ext}"
        output = Path(filepath)
        if not output.exists():
            output.parent.mkdir(exist_ok=True, parents=True)
            with output.open("wb") as f:
                f.write(resp.content)
        return filepath

if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("spreadsheet_path")
    parser.add_argument("image_path")
    parser.add_argument("json_path")
    args = parser.parse_args()

    events = []
    with open(args.spreadsheet_path, "r") as csvfile:
        reader = csv.DictReader(csvfile)
        for i, line in enumerate(reader, start=1):
            if line["EventType"]:
                event = process(args, line)
                print(i, event)
                events.append(event)

    output = Path(args.json_path)
    output.parent.mkdir(exist_ok=True, parents=True)
    with output.open("w", encoding="utf-8") as f:
        data = { "events": events }
        json.dump(data, f, ensure_ascii=False, indent=4)
