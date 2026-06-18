from __future__ import annotations

import argparse
import random
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


CANVAS_SIZE = (1920, 920)


def resolve_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def default_output_path() -> Path:
    return resolve_repo_root() / "src/yian-web/public/assets/yian-network-banner.png"


def pick_font(size: int, *, bold: bool = False) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    font_dir = Path("C:/Windows/Fonts")
    names = (
        ["msyhbd.ttc", "simhei.ttf", "msyh.ttc", "simsun.ttc"]
        if bold
        else ["msyh.ttc", "simhei.ttf", "simsun.ttc"]
    )
    for name in names:
        path = font_dir / name
        if path.exists():
            return ImageFont.truetype(str(path), size)
    return ImageFont.load_default()


def rounded(draw: ImageDraw.ImageDraw, box, outline, fill, width=3, radius=30) -> None:
    draw.rounded_rectangle(box, radius=radius, outline=outline, width=width, fill=fill)


def dashed_line(draw: ImageDraw.ImageDraw, points, color, width=3, dash=(14, 8)) -> None:
    for (x1, y1), (x2, y2) in zip(points, points[1:]):
        length = ((x2 - x1) ** 2 + (y2 - y1) ** 2) ** 0.5
        if length == 0:
            continue
        dx, dy = (x2 - x1) / length, (y2 - y1) / length
        pos = 0
        while pos < length:
            end = min(pos + dash[0], length)
            draw.line(
                (x1 + dx * pos, y1 + dy * pos, x1 + dx * end, y1 + dy * end),
                fill=color,
                width=width,
            )
            pos += dash[0] + dash[1]


def arrow_head(draw: ImageDraw.ImageDraw, x: int, y: int, direction: str, color) -> None:
    if direction == "right":
        points = [(x, y), (x - 14, y - 8), (x - 14, y + 8)]
    elif direction == "left":
        points = [(x, y), (x + 14, y - 8), (x + 14, y + 8)]
    elif direction == "down":
        points = [(x, y), (x - 8, y - 14), (x + 8, y - 14)]
    else:
        points = [(x, y), (x - 8, y + 14), (x + 8, y + 14)]
    draw.polygon(points, fill=color)


def render(output_path: Path) -> None:
    width, height = CANVAS_SIZE
    image = Image.new("RGB", CANVAS_SIZE, "#071b2c")
    draw = ImageDraw.Draw(image)

    for x in range(width):
        t = x / (width - 1)
        r = int(7 * (1 - t) + 5 * t)
        g = int(27 * (1 - t) + 82 * t)
        b = int(44 * (1 - t) + 70 * t)
        draw.line([(x, 0), (x, height)], fill=(r, g, b))

    random.seed(9)
    for _ in range(46):
        x1 = random.randint(-60, width + 60)
        y1 = random.randint(-40, height + 40)
        x2 = x1 + random.randint(-150, 150)
        y2 = y1 + random.randint(-120, 120)
        draw.line((x1, y1, x2, y2), fill=(38, 118, 132), width=2)

    title = pick_font(60, bold=True)
    h1 = pick_font(36, bold=True)
    h2 = pick_font(32, bold=True)
    body = pick_font(24)
    small = pick_font(20)
    tiny = pick_font(18)

    white = "#f7fbfd"
    muted = "#cce8ee"
    cyan = "#45e3f4"
    blue = "#3baec1"
    teal = "#2dbf9c"
    yellow = "#f0bd46"

    draw.text((84, 78), "易安 Yian", font=title, fill=white)

    rounded(draw, (60, 230, 490, 770), blue, (20, 45, 71), 3, 34)
    draw.text((104, 276), "Agent 平台", font=h1, fill=white)
    draw.text((104, 326), "pharos / OpenClaw / 第三方 Agent", font=body, fill=cyan)

    rounded(draw, (560, 230, 1180, 770), blue, (15, 60, 72), 3, 34)
    draw.text((604, 276), "三方云服务平台", font=h1, fill=white)
    draw.text((604, 326), "AWS / 华为云 / Vercel / 企业云", font=body, fill=cyan)

    rounded(draw, (1250, 230, 1860, 770), teal, (8, 77, 66), 3, 34)
    draw.text((1294, 276), "用户本地设备", font=h1, fill=white)
    draw.text((1294, 326), "有网络的助理 + 无网络的本地核心", font=body, fill=cyan)

    rounded(draw, (105, 405, 445, 605), blue, (20, 48, 76), 3, 26)
    draw.text((133, 435), "第三方 Agent", font=h2, fill=white)
    draw.text((133, 486), "通过 Skill 发起请求", font=body, fill=cyan)
    draw.text((133, 528), "不读取本地故事或密钥", font=small, fill=muted)

    rounded(draw, (620, 405, 1120, 605), blue, (16, 57, 72), 3, 26)
    draw.text((650, 435), "易安远程入口", font=h2, fill=white)
    draw.text((650, 486), "部署在三方云服务平台", font=body, fill=cyan)
    draw.text((650, 528), "接收请求 / 转发状态 / 提供下载绑定", font=small, fill=muted)

    rounded(draw, (1305, 385, 1795, 565), teal, (13, 70, 72), 3, 24)
    draw.text((1334, 414), "私人智能助理", font=h2, fill=white)
    draw.text((1334, 468), "联网接收请求，返回状态", font=body, fill=cyan)
    draw.text((1334, 510), "解释来源与风险，可辅助生成故事模板", font=small, fill=muted)

    rounded(draw, (1305, 610, 1795, 765), yellow, (34, 63, 54), 3, 22)
    draw.text((1334, 638), "StoryLock 本地核心", font=h2, fill=white)
    draw.text((1334, 692), "无网络服务，本地确认", font=body, fill=cyan)
    draw.text((1334, 730), "最小结果只回私人智能助理", font=tiny, fill=muted)
    draw.rounded_rectangle((1628, 684, 1752, 725), radius=14, outline=yellow, width=2, fill=(48, 65, 55))
    draw.text((1660, 693), "无网络", font=small, fill=yellow)

    dashed_line(draw, [(445, 505), (620, 505)], blue)
    arrow_head(draw, 620, 505, "right", blue)
    draw.text((474, 474), "Skill 调用", font=small, fill=cyan)

    dashed_line(draw, [(1120, 505), (1305, 472)], blue)
    arrow_head(draw, 1305, 472, "right", blue)
    arrow_head(draw, 1120, 505, "left", blue)
    draw.text((1130, 438), "双向通信 / 确认状态", font=small, fill=cyan)

    dashed_line(draw, [(1550, 565), (1550, 610)], yellow, dash=(10, 8))
    arrow_head(draw, 1550, 610, "down", yellow)
    arrow_head(draw, 1550, 565, "up", yellow)
    draw.text((1574, 582), "本地调用 / 最小结果", font=small, fill=yellow)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    image.save(output_path, format="PNG", optimize=True)
    print(f"saved {output_path} {image.size}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Render the Yian network banner PNG.")
    parser.add_argument(
        "--out",
        type=Path,
        default=default_output_path(),
        help="Output PNG path. Defaults to src/yian-web/public/assets/yian-network-banner.png.",
    )
    args = parser.parse_args()
    render(args.out)


if __name__ == "__main__":
    main()
