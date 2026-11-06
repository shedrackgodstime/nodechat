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

test("a wide desktop shows the section rail, chat list, and conversation pane", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  const texts = app.texts();
  expect(texts).toContain("NodeChat");
  expect(texts).toContain("Select a conversation");
  expect(texts).toContain("Ada");
});

test("a medium desktop keeps section navigation and chats in one sidebar", async () => {
  const app = await open(".", { platform: "linux", width: 1000, height: 800 });
  await app.press("Ada");
  expect(app.texts()).toContain("Contacts");
  expect(app.texts()).toContain("I pushed the new build — try it now.");
});

test("opening a chat on a desktop fills the pane", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("Ada");
  expect(app.texts()).toContain("I pushed the new build — try it now.");
});

test("desktop navigation reaches contacts and settings", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("Browse contacts");
  expect(app.texts()).toContain("YOUR TICKET");
  await app.press("Chats");
  await app.press("Contacts");
  expect(app.texts()).toContain("YOUR TICKET");
  await app.press("Settings");
  expect(app.texts()).toContain("Notifications");
});

test("desktop chat search filters conversations", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.type("Search", "Mira");
  expect(app.texts()).toContain("Mira");
  expect(app.texts()).not.toContain("Ada");
});

test("new chat opens contacts and selecting a peer opens their conversation", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("New chat");
  expect(app.texts()).toContain("YOUR TICKET");
  await app.press("Open chat with Mira");
  expect(app.texts()).toContain("Sent you the notes.");
});

test("sending a message adds it to the conversation and chat preview", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("Ada");
  await app.type("Message", "Looks great, thanks!");
  await app.press("Send");
  expect(app.texts()).toContain("Looks great, thanks!");
});

test("sending after switching chats updates the selected chat only", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("Ada");
  await app.press("Mira");
  await app.type("Message", "Here are the session notes.");
  await app.press("Send");
  const mentions = () =>
    app.texts().filter((text) => text === "Here are the session notes.").length;
  expect(mentions()).toBe(2);
  await app.press("Ada");
  expect(app.texts()).toContain("I pushed the new build — try it now.");
  expect(mentions()).toBe(1);
  await app.press("Mira");
  expect(mentions()).toBe(2);
});

test("opening a chat on a phone pushes it", async () => {
  const app = await open("."); // iOS, 390 × 844
  await app.press("Ada");
  // The conversation screens shows the peer's messages.
  expect(app.texts()).toContain("I pushed the new build — try it now.");
  // The chat list is behind the pushed screen.
  await app.back();
  expect(app.texts()).toContain("Chats");
});
