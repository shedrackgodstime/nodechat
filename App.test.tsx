// The Rust root tested from TypeScript: `nsh test` runs this file in a
// headless runner it builds with the crate, and `open(".")` starts the root
// (`nsh.json` "root").
//
// The shell is responsive: a phone shows the chat list with a tab bar (and
// opening a chat pushes it); a desktop shows the sidebar beside the
// conversation pane. The runner opens a phone by default; `platform` and
// `width` choose otherwise.
import { expect, open, test } from "nsh/test";

test("a phone shows the chat list with the section tabs", async () => {
  const app = await open("."); // iOS, 390 × 844
  const texts = app.texts();
  expect(texts).toContain("Chats");
  expect(texts).toContain("Ada");
  expect(texts).toContain("Contacts");
  expect(texts).toContain("Settings");
});

test("a desktop shows the sidebar beside the conversation pane", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  const texts = app.texts();
  expect(texts).toContain("NodeChat");
  // The empty pane before a chat is chosen.
  expect(texts).toContain("Select a conversation");
  expect(texts).toContain("Ada");
});

test("opening a chat on a desktop fills the pane", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("Ada");
  expect(app.texts()).toContain("Sounds good, see you then.");
});

test("opening a chat on a phone pushes it", async () => {
  const app = await open("."); // iOS, 390 × 844
  await app.press("Ada");
  // The conversation screens shows the peer's messages.
  expect(app.texts()).toContain("Sounds good, see you then.");
  // The chat list is behind the pushed screen.
  await app.back();
  expect(app.texts()).toContain("Chats");
});
