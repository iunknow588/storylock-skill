from __future__ import annotations

import argparse
import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont


CANVAS_SIZE = (2400, 1350)


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


def lerp_color(a: tuple[int, int, int], b: tuple[int, int, int], t: float) -> tuple[int, int, int]:
    return (
        int(a[0] * (1 - t) + b[0] * t),
        int(a[1] * (1 - t) + b[1] * t),
        int(a[2] * (1 - t) + b[2] * t),
    )


def draw_background(image: Image.Image) -> None:
    draw = ImageDraw.Draw(image)
    width, height = image.size
    for y in range(height):
        t = y / max(height - 1, 1)
        draw.line((0, y, width, y), fill=lerp_color((7, 13, 24), (14, 36, 49), t))

    for x in range(-220, width, 150):
        draw.line((x, 0, x + 310, height), fill=(28, 70, 82), width=1)
    for y in range(120, height, 120):
        draw.line((0, y, width, y), fill=(22, 54, 66), width=1)

    glow = Image.new("RGBA", image.size, (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow)
    glow_draw.ellipse((1480, -180, 2540, 720), fill=(48, 126, 147, 58))
    glow_draw.ellipse((-260, 760, 720, 1460), fill=(66, 98, 167, 48))
    image.alpha_composite(glow.filter(ImageFilter.GaussianBlur(54)))


def rounded(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    *,
    fill: str | tuple[int, int, int, int],
    outline: str | tuple[int, int, int, int],
    width: int = 2,
    radius: int = 24,
) -> None:
    draw.rounded_rectangle(box, radius=radius, fill=fill, outline=outline, width=width)


def text_size(draw: ImageDraw.ImageDraw, text: str, font: ImageFont.ImageFont) -> tuple[int, int]:
    if not text:
        return 0, 0
    box = draw.textbbox((0, 0), text, font=font)
    return box[2] - box[0], box[3] - box[1]


def wrap_text(draw: ImageDraw.ImageDraw, text: str, font: ImageFont.ImageFont, max_width: int) -> list[str]:
    lines: list[str] = []
    for paragraph in text.split("\n"):
        current = ""
        for ch in paragraph:
            trial = current + ch
            if text_size(draw, trial, font)[0] <= max_width:
                current = trial
            else:
                if current:
                    lines.append(current)
                current = ch
        if current:
            lines.append(current)
    return lines


def draw_text_block(
    draw: ImageDraw.ImageDraw,
    xy: tuple[int, int],
    text: str,
    *,
    font: ImageFont.ImageFont,
    fill: str,
    max_width: int,
    line_gap: int = 7,
) -> int:
    x, y = xy
    lines = wrap_text(draw, text, font, max_width)
    line_height = text_size(draw, "中Ag", font)[1] + 3
    for i, line in enumerate(lines):
        draw.text((x, y + i * (line_height + line_gap)), line, font=font, fill=fill)
    return len(lines) * line_height + max(0, len(lines) - 1) * line_gap


def draw_arrow(
    draw: ImageDraw.ImageDraw,
    start: tuple[int, int],
    end: tuple[int, int],
    *,
    color: str,
    width: int = 5,
) -> None:
    draw.line((start, end), fill=color, width=width)
    x1, y1 = start
    x2, y2 = end
    angle = math.atan2(y2 - y1, x2 - x1)
    length = 18
    spread = 0.55
    p1 = (x2, y2)
    p2 = (int(x2 - length * math.cos(angle - spread)), int(y2 - length * math.sin(angle - spread)))
    p3 = (int(x2 - length * math.cos(angle + spread)), int(y2 - length * math.sin(angle + spread)))
    draw.polygon((p1, p2, p3), fill=color)


def draw_badge(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    *,
    title: str,
    body: str,
    accent: str,
    fonts: dict[str, ImageFont.ImageFont],
    colors: dict[str, str],
) -> None:
    x1, y1, x2, y2 = box
    rounded(draw, box, fill=colors["chip"], outline=colors["chip_line"], width=2, radius=18)
    draw.rounded_rectangle((x1 + 16, y1 + 18, x1 + 22, y2 - 18), radius=3, fill=accent)
    draw.text((x1 + 36, y1 + 16), title, font=fonts["badge_title"], fill=colors["white"])
    draw.text((x1 + 36, y1 + 52), body, font=fonts["badge_body"], fill=colors["muted"])


def draw_layer_card(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    *,
    tag: str,
    title: str,
    subtitle: str,
    items: list[str],
    boundary: str,
    accent: str,
    fonts: dict[str, ImageFont.ImageFont],
    colors: dict[str, str],
) -> None:
    x1, y1, x2, y2 = box
    rounded(draw, box, fill=colors["panel"], outline=accent, width=3, radius=26)
    draw.rounded_rectangle((x1 + 24, y1 + 22, x1 + 124, y1 + 60), radius=16, fill=accent)
    draw.text((x1 + 42, y1 + 30), tag, font=fonts["tag"], fill=colors["ink"])
    draw.text((x1 + 24, y1 + 82), title, font=fonts["card_title"], fill=colors["white"])
    draw.text((x1 + 24, y1 + 126), subtitle, font=fonts["small"], fill=colors["muted2"])

    current_y = y1 + 178
    for item in items:
        draw.ellipse((x1 + 28, current_y + 9, x1 + 40, current_y + 21), fill=accent)
        used = draw_text_block(
            draw,
            (x1 + 56, current_y),
            item,
            font=fonts["body"],
            fill=colors["muted"],
            max_width=x2 - x1 - 84,
            line_gap=5,
        )
        current_y += used + 18

    boundary_box = (x1 + 20, y2 - 116, x2 - 20, y2 - 22)
    rounded(draw, boundary_box, fill=colors["boundary"], outline=accent, width=2, radius=18)
    draw.text((x1 + 38, y2 - 100), "关键边界", font=fonts["mini_bold"], fill=colors["white"])
    draw_text_block(
        draw,
        (x1 + 38, y2 - 66),
        boundary,
        font=fonts["mini"],
        fill=colors["muted"],
        max_width=x2 - x1 - 76,
        line_gap=3,
    )


def draw_service_chip(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    *,
    title: str,
    body: str,
    accent: str,
    fonts: dict[str, ImageFont.ImageFont],
    colors: dict[str, str],
) -> None:
    x1, y1, x2, y2 = box
    rounded(draw, box, fill=colors["service"], outline=colors["service_line"], width=2, radius=18)
    draw.ellipse((x1 + 18, y1 + 18, x1 + 34, y1 + 34), fill=accent)
    draw.text((x1 + 46, y1 + 12), title, font=fonts["service_title"], fill=colors["white"])
    draw_text_block(
        draw,
        (x1 + 46, y1 + 42),
        body,
        font=fonts["service_body"],
        fill=colors["muted"],
        max_width=x2 - x1 - 62,
        line_gap=3,
    )


def draw_bottom_tile(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    *,
    title: str,
    body: str,
    accent: str,
    fonts: dict[str, ImageFont.ImageFont],
    colors: dict[str, str],
) -> None:
    x1, y1, x2, y2 = box
    rounded(draw, box, fill=colors["tile"], outline=colors["tile_line"], width=2, radius=18)
    draw.text((x1 + 18, y1 + 16), title, font=fonts["tile_title"], fill=accent)
    draw_text_block(
        draw,
        (x1 + 18, y1 + 50),
        body,
        font=fonts["mini"],
        fill=colors["muted"],
        max_width=x2 - x1 - 36,
        line_gap=3,
    )


def render(output_path: Path) -> None:
    image = Image.new("RGBA", CANVAS_SIZE, (0, 0, 0, 255))
    draw_background(image)
    draw = ImageDraw.Draw(image)

    fonts = {
        "title": pick_font(66, bold=True),
        "subtitle": pick_font(28),
        "badge_title": pick_font(27, bold=True),
        "badge_body": pick_font(21),
        "tag": pick_font(24, bold=True),
        "card_title": pick_font(39, bold=True),
        "body": pick_font(24),
        "small": pick_font(23),
        "mini": pick_font(20),
        "mini_bold": pick_font(20, bold=True),
        "section": pick_font(34, bold=True),
        "service_title": pick_font(24, bold=True),
        "service_body": pick_font(20),
        "tile_title": pick_font(24, bold=True),
    }

    colors = {
        "ink": "#08111d",
        "white": "#f5fbff",
        "muted": "#c8dbe5",
        "muted2": "#91afbd",
        "panel": "#0e2334",
        "boundary": "#142c3e",
        "chip": "#10283a",
        "chip_line": "#315d71",
        "service": "#14283c",
        "service_line": "#4b7386",
        "tile": "#0f2434",
        "tile_line": "#2d5165",
        "cyan": "#5eead4",
        "blue": "#64b5ff",
        "amber": "#f4c95d",
        "green": "#89e082",
        "rose": "#f28ba8",
    }

    # Trust boundaries behind the main pipeline.
    rounded(draw, (72, 330, 1158, 960), fill=(55, 103, 117, 38), outline="#30586a", width=2, radius=38)
    draw.text((104, 350), "本地设备信任域", font=fonts["mini_bold"], fill=colors["muted2"])
    rounded(draw, (1196, 330, 2330, 960), fill=(54, 88, 143, 36), outline="#42617d", width=2, radius=38)
    draw.text((1228, 350), "华为云远程网关与分发域", font=fonts["mini_bold"], fill=colors["muted2"])

    draw.text((92, 70), "易安 Yian / StoryLock 技术架构图", font=fonts["title"], fill=colors["white"])
    draw.text(
        (96, 154),
        "本地优先、三层隔离、用内涵记忆取代机械记忆，构建面向 Agent 的安全授权体系。",
        font=fonts["subtitle"],
        fill=colors["muted"],
    )

    draw_badge(
        draw,
        (92, 224, 516, 326),
        title="核心理念",
        body="故事记忆替代传统密码",
        accent=colors["cyan"],
        fonts=fonts,
        colors=colors,
    )
    draw_badge(
        draw,
        (548, 224, 972, 326),
        title="交互模型",
        body="24 题题集抽取 9 宫格挑战",
        accent=colors["blue"],
        fonts=fonts,
        colors=colors,
    )
    draw_badge(
        draw,
        (1004, 224, 1428, 326),
        title="安全原则",
        body="长期秘密留本地，云侧只做网关",
        accent=colors["amber"],
        fonts=fonts,
        colors=colors,
    )

    layer_boxes = [
        (104, 420, 564, 910),
        (650, 420, 1110, 910),
        (1228, 420, 1688, 910),
    ]
    draw_layer_card(
        draw,
        layer_boxes[0],
        tag="第一层",
        title="故事处理层",
        subtitle="用户故事 -> 题集与加密故事文件",
        items=[
            "故事草稿、润色、题集强度评估",
            "生成 24 题题集与加密故事文件",
            "本地文件系统保存故事主档",
        ],
        boundary="不读取受保护对象，不签发授权。",
        accent=colors["cyan"],
        fonts=fonts,
        colors=colors,
    )
    draw_layer_card(
        draw,
        layer_boxes[1],
        tag="第二层",
        title="本地授权层",
        subtitle="九宫格挑战 -> 短时 session",
        items=[
            "对象强度策略、九宫格挑战、本地授权",
            "校验答案摘要，签发短时 session",
            "SQLite 记录 nonce、防重放与审计",
        ],
        boundary="不返回长期秘密，不直接执行远程请求。",
        accent=colors["blue"],
        fonts=fonts,
        colors=colors,
    )
    draw_layer_card(
        draw,
        layer_boxes[2],
        tag="第三层",
        title="远程网关层",
        subtitle="EIP-712 请求 -> 脱敏结果",
        items=[
            "Web API、EIP-712 解析、脱敏中间件",
            "对 Agent 初步验签并转发本地宿主",
            "只返回脱敏结果与审计元数据",
        ],
        boundary="不持有私钥/密码，不透传敏感字段。",
        accent=colors["amber"],
        fonts=fonts,
        colors=colors,
    )

    draw_arrow(draw, (564, 650), (650, 650), color=colors["blue"])
    draw.text((584, 606), "题集", font=fonts["mini"], fill=colors["muted"])
    draw_arrow(draw, (1110, 650), (1228, 650), color=colors["amber"])
    draw.text((1146, 606), "授权", font=fonts["mini"], fill=colors["muted"])

    cloud_box = (1760, 420, 2292, 910)
    rounded(draw, cloud_box, fill="#122638", outline=colors["green"], width=3, radius=28)
    draw.text((1790, 452), "华为云能力映射", font=fonts["section"], fill=colors["white"])
    draw.text((1792, 500), "第三层运行与软件分发承载", font=fonts["small"], fill=colors["muted2"])

    services = [
        ("APIG", "统一路由、HTTPS、限流", colors["green"]),
        ("FunctionGraph", "Serverless 网关计算", colors["amber"]),
        ("OBS", "网站与安装包分发", colors["blue"]),
        ("CloudTable", "可选审计归档", colors["rose"]),
        ("KMS", "公钥与配置加密", colors["cyan"]),
    ]
    service_positions = [
        (1792, 560, 2262, 626),
        (1792, 640, 2262, 706),
        (1792, 720, 2262, 786),
        (1792, 800, 2018, 866),
        (2034, 800, 2262, 866),
    ]
    for (title, body, accent), box in zip(services, service_positions):
        draw_service_chip(draw, box, title=title, body=body, accent=accent, fonts=fonts, colors=colors)

    draw_arrow(draw, (1688, 650), (1760, 650), color=colors["green"])
    draw.text((1702, 606), "网关", font=fonts["mini"], fill=colors["muted"])

    # Bottom rail: security mechanisms and route maturity.
    bottom_title_y = 994
    draw.text((92, bottom_title_y), "安全机制与路线成熟度", font=fonts["section"], fill=colors["white"])
    draw.text(
        (452, bottom_title_y + 8),
        "成熟标准优先，无自研密码算法；当前本地闭环完成，近期替换为华为云 FunctionGraph + OBS + APIG。",
        font=fonts["small"],
        fill=colors["muted2"],
    )

    tiles = [
        ("密钥派生链", "masterSalt -> rootKey -> workKey\n-> objectKey，长期有效且可随故事更新。", colors["cyan"]),
        ("认证加密", "AES-256-GCM + 96-bit nonce，答案摘要使用 HMAC-SHA256。", colors["blue"]),
        ("防重放", "requestId、nonce、expiry 三重校验，默认 24 小时去重窗口。", colors["amber"]),
        ("递归脱敏", "none / partial / full 三级策略，外部只见最小必要结果。", colors["green"]),
        ("演进路径", "当前本地三层闭环，近期华为云部署，中期探索 ZKP，远期 MPC/HE。", colors["rose"]),
    ]
    tile_width = 430
    gap = 26
    start_x = 92
    for i, (title, body, accent) in enumerate(tiles):
        x = start_x + i * (tile_width + gap)
        draw_bottom_tile(
            draw,
            (x, 1070, x + tile_width, 1204),
            title=title,
            body=body,
            accent=accent,
            fonts=fonts,
            colors=colors,
        )

    footer = "StoryLock 以故事内涵为魂、成熟密码学为基、华为云 Serverless 为翼，构建本地优先的 Agent 安全授权体系。"
    draw.text((92, 1264), footer, font=fonts["mini"], fill=colors["muted2"])

    output_path.parent.mkdir(parents=True, exist_ok=True)
    image.convert("RGB").save(output_path, format="PNG", optimize=True)
    print(f"saved {output_path} {image.size}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Render the Yian StoryLock architecture banner.")
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
