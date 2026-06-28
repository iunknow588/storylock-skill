from __future__ import annotations

import argparse
import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont


CANVAS_SIZE = (2400, 1350)


def resolve_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def default_output_path() -> Path:
    return resolve_repo_root() / "src/yian-web/public/assets/yian-business-flow.png"


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
        draw.line((0, y, width, y), fill=lerp_color((7, 13, 24), (13, 38, 50), t))

    for x in range(-260, width, 170):
        draw.line((x, 0, x + 320, height), fill=(25, 67, 79), width=1)
    for y in range(120, height, 130):
        draw.line((0, y, width, y), fill=(20, 54, 65), width=1)

    glow = Image.new("RGBA", image.size, (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow)
    glow_draw.ellipse((1340, -180, 2620, 780), fill=(58, 117, 170, 50))
    glow_draw.ellipse((-320, 680, 880, 1480), fill=(60, 148, 128, 42))
    image.alpha_composite(glow.filter(ImageFilter.GaussianBlur(58)))


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
    line_gap: int = 6,
) -> int:
    x, y = xy
    lines = wrap_text(draw, text, font, max_width)
    line_height = text_size(draw, "中Ag", font)[1] + 4
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
    length = 20
    spread = 0.55
    p1 = (x2, y2)
    p2 = (int(x2 - length * math.cos(angle - spread)), int(y2 - length * math.sin(angle - spread)))
    p3 = (int(x2 - length * math.cos(angle + spread)), int(y2 - length * math.sin(angle + spread)))
    draw.polygon((p1, p2, p3), fill=color)


def draw_flow_card(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    *,
    number: str,
    title: str,
    body: str,
    note: str,
    accent: str,
    fonts: dict[str, ImageFont.ImageFont],
    colors: dict[str, str],
) -> None:
    x1, y1, x2, y2 = box
    rounded(draw, box, fill=colors["panel"], outline=accent, width=3, radius=28)
    draw.ellipse((x1 + 24, y1 + 24, x1 + 78, y1 + 78), fill=accent)
    tw, th = text_size(draw, number, fonts["number"])
    draw.text((x1 + 51 - tw / 2, y1 + 51 - th / 2 - 2), number, font=fonts["number"], fill=colors["ink"])
    draw.text((x1 + 96, y1 + 24), title, font=fonts["card_title"], fill=colors["white"])
    draw_text_block(
        draw,
        (x1 + 28, y1 + 96),
        body,
        font=fonts["body"],
        fill=colors["muted"],
        max_width=x2 - x1 - 56,
        line_gap=6,
    )
    rounded(draw, (x1 + 24, y2 - 78, x2 - 24, y2 - 24), fill=colors["note"], outline=accent, width=2, radius=18)
    draw.text((x1 + 42, y2 - 62), note, font=fonts["note"], fill=colors["muted2"])


def draw_stage_band(
    draw: ImageDraw.ImageDraw,
    box: tuple[int, int, int, int],
    *,
    title: str,
    accent: str,
    fonts: dict[str, ImageFont.ImageFont],
    colors: dict[str, str],
) -> None:
    rounded(draw, box, fill=colors["band"], outline=colors["band_line"], width=2, radius=34)
    draw.text((box[0] + 28, box[1] + 22), title, font=fonts["band"], fill=accent)


def draw_chip(
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
    draw.rounded_rectangle((x1 + 18, y1 + 18, x1 + 24, y2 - 18), radius=3, fill=accent)
    draw.text((x1 + 42, y1 + 16), title, font=fonts["chip_title"], fill=colors["white"])
    draw_text_block(
        draw,
        (x1 + 42, y1 + 52),
        body,
        font=fonts["chip_body"],
        fill=colors["muted"],
        max_width=x2 - x1 - 62,
        line_gap=3,
    )


def render(output_path: Path) -> None:
    image = Image.new("RGBA", CANVAS_SIZE, (0, 0, 0, 255))
    draw_background(image)
    draw = ImageDraw.Draw(image)

    fonts = {
        "title": pick_font(66, bold=True),
        "subtitle": pick_font(28),
        "band": pick_font(24, bold=True),
        "number": pick_font(26, bold=True),
        "card_title": pick_font(31, bold=True),
        "body": pick_font(21),
        "note": pick_font(18, bold=True),
        "section": pick_font(34, bold=True),
        "chip_title": pick_font(24, bold=True),
        "chip_body": pick_font(20),
        "footer": pick_font(21),
    }

    colors = {
        "ink": "#08111d",
        "white": "#f5fbff",
        "muted": "#c8dbe5",
        "muted2": "#9eb8c7",
        "panel": "#0e2334",
        "note": "#142c3e",
        "band": (48, 102, 114, 42),
        "band_line": "#315d71",
        "chip": "#10283a",
        "chip_line": "#315d71",
        "cyan": "#5eead4",
        "blue": "#64b5ff",
        "amber": "#f4c95d",
        "green": "#89e082",
        "rose": "#f28ba8",
        "violet": "#b69cff",
    }

    draw.text((92, 70), "易安 Yian / StoryLock 业务流程图", font=fonts["title"], fill=colors["white"])
    draw.text(
        (96, 154),
        "从私密故事到九宫格挑战，再到短时授权与脱敏返回，形成本地优先的安全授权闭环。",
        font=fonts["subtitle"],
        fill=colors["muted"],
    )

    draw_stage_band(draw, (56, 220, 1156, 784), title="本地准备阶段", accent=colors["cyan"], fonts=fonts, colors=colors)
    draw_stage_band(draw, (1244, 220, 2344, 784), title="授权访问阶段", accent=colors["amber"], fonts=fonts, colors=colors)
    draw_stage_band(draw, (72, 816, 2328, 1156), title="返回、审计与持续防护", accent=colors["green"], fonts=fonts, colors=colors)

    top_cards = [
        (
            (104, 326, 424, 704),
            "01",
            "创建私密故事",
            "用户在本地设备中输入私密故事。系统围绕故事语义形成可记忆、可追问、可更新的身份内涵。",
            "长期记忆来自用户本人",
            colors["cyan"],
        ),
        (
            (456, 326, 776, 704),
            "02",
            "生成题集与主档",
            "系统整理故事内容，生成 24 题题集，并完成强度评估。题集用于后续动态抽题，不作为普通明文密码使用。",
            "24 题题集 + 加密故事文件",
            colors["blue"],
        ),
        (
            (808, 326, 1128, 704),
            "03",
            "本地加密保存",
            "故事主档、题集摘要和关键密钥材料保留在本地设备。受保护对象使用成熟密码学机制完成加密。",
            "长期秘密不上传云端",
            colors["green"],
        ),
        (
            (1306, 326, 1626, 704),
            "04",
            "Agent 发起请求",
            "外部 Agent 请求访问受保护对象。请求进入远程网关，完成基础校验、结构包装和来源检查。",
            "云端只承担入口与网关",
            colors["amber"],
        ),
        (
            (1658, 326, 1978, 704),
            "05",
            "触发九宫格挑战",
            "系统依据对象类型和安全等级，从 24 题题集中动态抽取问题，生成本次挑战。",
            "每次题目组合可变化",
            colors["violet"],
        ),
        (
            (2010, 326, 2330, 704),
            "06",
            "用户本地确认",
            "用户基于故事记忆回答问题，系统校验答案摘要，通过后生成短时授权结果。",
            "只签发短时授权",
            colors["rose"],
        ),
    ]

    for card in top_cards:
        draw_flow_card(
            draw,
            card[0],
            number=card[1],
            title=card[2],
            body=card[3],
            note=card[4],
            accent=card[5],
            fonts=fonts,
            colors=colors,
        )

    for start_x, end_x, color in [
        (424, 456, colors["blue"]),
        (776, 808, colors["green"]),
        (1128, 1306, colors["amber"]),
        (1626, 1658, colors["violet"]),
        (1978, 2010, colors["rose"]),
    ]:
        draw_arrow(draw, (start_x, 515), (end_x, 515), color=color, width=5)

    bottom_cards = [
        (
            (104, 878, 552, 1134),
            "07",
            "脱敏返回",
            "授权结果进入远程网关后进行递归脱敏。外部只收到必要状态、摘要或审计元数据。",
            "敏感字段不透传",
            colors["amber"],
        ),
        (
            (604, 878, 1052, 1134),
            "08",
            "完成访问",
            "Agent 根据脱敏授权结果执行后续操作。长期私钥、原始答案与故事密钥仍留在本地。",
            "最小必要结果返回",
            colors["green"],
        ),
        (
            (1104, 878, 1552, 1134),
            "09",
            "审计与防重放",
            "系统记录 requestId、nonce、expiry 与授权状态，用于去重、防重放和事后追溯。",
            "请求可追踪，秘密不暴露",
            colors["blue"],
        ),
        (
            (1604, 878, 2052, 1134),
            "10",
            "策略持续调整",
            "若出现高频失败、异常访问或高风险对象，系统可提高挑战强度或要求重新确认。",
            "安全策略动态收紧",
            colors["rose"],
        ),
    ]

    for card in bottom_cards:
        draw_flow_card(
            draw,
            card[0],
            number=card[1],
            title=card[2],
            body=card[3],
            note=card[4],
            accent=card[5],
            fonts=fonts,
            colors=colors,
        )

    draw_arrow(draw, (2170, 704), (2170, 812), color=colors["amber"], width=5)
    draw_arrow(draw, (2170, 812), (328, 812), color=colors["amber"], width=5)
    draw_arrow(draw, (328, 812), (328, 878), color=colors["amber"], width=5)

    for start_x, end_x, color in [
        (552, 604, colors["green"]),
        (1052, 1104, colors["blue"]),
        (1552, 1604, colors["rose"]),
    ]:
        draw_arrow(draw, (start_x, 1006), (end_x, 1006), color=color, width=5)

    draw.text((92, 1206), "业务价值", font=fonts["section"], fill=colors["white"])
    chips = [
        ((256, 1190, 712, 1288), "用户体验", "记住自己的故事，而不是记住复杂密码。", colors["cyan"]),
        ((748, 1190, 1204, 1288), "安全边界", "长期秘密留在本地，云端只处理网关与脱敏结果。", colors["amber"]),
        ((1240, 1190, 1696, 1288), "Agent 适配", "支持外部 Agent 请求，同时保留人工确认入口。", colors["blue"]),
        ((1732, 1190, 2188, 1288), "持续防护", "挑战、审计、防重放和策略调整形成闭环。", colors["green"]),
    ]
    for box, title, body, accent in chips:
        draw_chip(draw, box, title=title, body=body, accent=accent, fonts=fonts, colors=colors)

    footer = "StoryLock 将身份确认、对象授权、远程网关和审计防护拆分为清晰流程，让 Agent 访问在可控边界内完成。"
    draw.text((92, 1310), footer, font=fonts["footer"], fill=colors["muted2"])

    output_path.parent.mkdir(parents=True, exist_ok=True)
    image.convert("RGB").save(output_path, format="PNG", optimize=True)
    print(f"saved {output_path} {image.size}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Render the Yian StoryLock business flow image.")
    parser.add_argument(
        "--out",
        type=Path,
        default=default_output_path(),
        help="Output PNG path. Defaults to src/yian-web/public/assets/yian-business-flow.png.",
    )
    args = parser.parse_args()
    render(args.out)


if __name__ == "__main__":
    main()
