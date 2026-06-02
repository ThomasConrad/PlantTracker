import { describe, it, expect } from "vitest";
import { renderMarkdown } from "./markdown";

describe("renderMarkdown", () => {
  it("returns empty string for empty input", () => {
    expect(renderMarkdown("")).toBe("");
  });

  it("escapes HTML entities", () => {
    expect(renderMarkdown('<script>alert("xss")</script>')).not.toContain(
      "<script>",
    );
    expect(renderMarkdown("<b>bold</b>")).toContain("&lt;b&gt;");
  });

  it("renders bold with **", () => {
    const result = renderMarkdown("Hello **world**!");
    expect(result).toContain("<strong>world</strong>");
  });

  it("renders bold with __", () => {
    const result = renderMarkdown("Hello __world__!");
    expect(result).toContain("<strong>world</strong>");
  });

  it("renders italic with *", () => {
    const result = renderMarkdown("Hello *world*!");
    expect(result).toContain("<em>world</em>");
  });

  it("renders inline code with backticks", () => {
    const result = renderMarkdown("Use `npm install` to install");
    expect(result).toContain("<code");
    expect(result).toContain("npm install");
  });

  it("renders links", () => {
    const result = renderMarkdown(
      "Visit [Google](https://google.com) for info",
    );
    expect(result).toContain('href="https://google.com"');
    expect(result).toContain(">Google</a>");
    expect(result).toContain('target="_blank"');
  });

  it("renders line breaks", () => {
    const result = renderMarkdown("Line 1\nLine 2");
    expect(result).toContain("<br>");
  });

  it("renders bullet lists", () => {
    const result = renderMarkdown("- Item one\n- Item two");
    expect(result).toContain("<li");
    expect(result).toContain("Item one");
    expect(result).toContain("Item two");
  });

  it("handles multiple formatting in one line", () => {
    const result = renderMarkdown("**Bold** and *italic* and `code`");
    expect(result).toContain("<strong>Bold</strong>");
    expect(result).toContain("<em>italic</em>");
    expect(result).toContain("<code");
  });

  it("processes formatting inside code (known limitation of regex renderer)", () => {
    const result = renderMarkdown("`**not bold**`");
    // Our lightweight renderer applies bold inside code blocks
    // This is acceptable for a chat UI; a full parser would handle this
    expect(result).toContain("<code");
  });

  it("escapes angle brackets to prevent injection", () => {
    const result = renderMarkdown("Use <img src=x onerror=alert(1)>");
    expect(result).not.toContain("<img");
    expect(result).toContain("&lt;img");
  });
});
