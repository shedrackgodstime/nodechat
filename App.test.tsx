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
  expect(app.texts()).toContain("New conversation");
  await app.press("Back");
  await app.press("Chats");
  await app.press("Contacts");
  expect(app.texts()).toContain("People");
  await app.press("Settings");
  expect(app.texts()).toContain("Notifications");
});

test("the People directory contains searchable contacts and groups", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("Contacts");
  expect(app.texts()).toContain("Groups");
  expect(app.texts()).toContain("Contacts");
  expect(app.texts()).toContain("Requests");
  expect(app.texts()).toContain("Design Room");
  expect(app.texts()).toContain("Ada");
  const adaRowsBeforeSearch = app.texts().filter((text) => text === "Ada").length;
  await app.type("Search people", "Mira");
  expect(app.texts()).toContain("Mira");
  expect(app.texts().filter((text) => text === "Ada").length).toBeLessThan(adaRowsBeforeSearch);
  await app.press("Start conversation");
  expect(app.texts()).toContain("New conversation");
  expect(app.texts()).toContain("Connect by ticket");
  expect(app.texts()).not.toContain("Search people");
  await app.press("Connect by ticket");
  expect(app.texts()).toContain("Connect by ticket");
  expect(app.texts()).toContain("Contact ticket or link");
});

test("the People page previews a ticket before saving an outgoing request", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("Contacts");
  await app.press("New chat");
  await app.press("Connect by ticket");
  await app.type("Contact ticket or link", "nodechat://contact/mira-1234");
  await app.type("Optional message", "We met at study group.");
  await app.press("Preview ticket");
  expect(app.texts()).toContain("Mira · contact preview");
  await app.press("Send connection request");
  expect(app.texts()).toContain(
    "Request saved on this device. The peer must accept before a connection is established.",
  );
  await app.press("Back");
  await app.press("Contacts");
  expect(app.texts()).toContain("Mira");
  expect(app.texts()).toContain("Pending · sent");
  expect(app.texts()).toContain("Cancel Mira request");
  await app.press("Requests");
  expect(app.texts()).toContain("We met at study group.");
  expect(app.texts()).toContain("Waiting for the peer to accept this connection request.");
  await app.press("Cancel request");
  expect(app.texts()).not.toContain("We met at study group.");
});

test("incoming peer requests can be accepted or declined", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("Contacts");
  await app.press("Requests");
  await app.press("Accept Noor");
  expect(app.texts()).toContain("Noor");
  expect(app.texts()).not.toContain("Accept Noor");
  await app.press("Decline Design Critique");
  expect(app.texts()).not.toContain("Design Critique");
});

test("accepting a group request adds the group only after explicit approval", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("Contacts");
  await app.press("Requests");
  await app.press("Join Design Critique");
  expect(app.texts()).toContain("Design Critique");
  expect(app.texts()).toContain("2 members · group conversation");
  await app.press("Group details");
  expect(app.texts()).toContain("New members need an admin's approval.");
});

test("accepting a group request opens the group on a phone", async () => {
  const app = await open(".");
  await app.press("Contacts");
  await app.press("Requests");
  await app.press("Join Design Critique");
  expect(app.texts()).toContain("Design Critique");
  expect(app.texts()).toContain("2 members · group conversation");
});

test("a group can be created before choosing contacts and opened immediately", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("Contacts");
  await app.press("New chat");
  await app.press("Create a group");
  expect(app.texts()).toContain("Group name");
  expect(app.texts()).not.toContain("Choose people");
  await app.type("Group name", "Saturday Study");
  await app.press("Create group");
  expect(app.texts()).toContain("Saturday Study");
  expect(app.texts()).toContain("1 member · group conversation");
  expect(app.texts()).toContain("Your group is ready");
  expect(app.texts()).toContain("Copy invite ticket");
  await app.press("Copy group ticket");
  expect(app.clipboard()).toContain("nodechat://group/");
  await app.press("Invite people");
  expect(app.texts()).toContain("Group info");
  expect(app.texts()).not.toContain("Group delivery is not active · messages are only simulated locally.");
  expect(app.texts()).toContain("New members need an admin's approval.");
  expect(app.texts()).toContain("Members · 1");
  expect(app.texts()).toContain("Rotate invite ticket");
  await app.press("Invite Ada");
  expect(app.texts()).toContain("Invitation sent to Ada.");
  await app.press("Rotate invite ticket");
  expect(app.texts()).toContain("The previous ticket can no longer be used for new joins.");
  await app.press("Back to group");
  expect(app.texts()).toContain("Your group is ready");
  await app.press("Chats");
  await app.press("Ada");
  expect(app.texts()).toContain("You invited them to join Saturday Study");
  await app.press("Request to join Saturday Study");
  expect(app.texts()).toContain("Join request sent · waiting for a group admin");
});

test("a group ticket preview creates an approval request rather than joining immediately", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("New chat");
  await app.press("Join with a group ticket");
  await app.type("Group ticket or link", "nodechat://group/study-circle-1234");
  await app.press("Preview group ticket");
  expect(app.texts()).toContain("Study Circle · group preview");
  await app.press("Request to join");
  expect(app.texts().some((text) => text.includes("Join request saved."))).toBe(true);
  expect(app.texts()).toContain(
    "Join request saved. You are not a member until an admin approves it.",
  );
});

test("an approval-required invitation creates a join request instead of joining immediately", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("Ada");
  expect(app.texts()).toContain("Private group invitation");
  expect(app.texts()).toContain("Study Circle");
  await app.press("Request to join Study Circle");
  expect(app.texts()).toContain("Join request sent · waiting for a group admin");
  await app.press("Contacts");
  await app.press("Requests");
  expect(app.texts()).toContain("Waiting for a group admin to approve your request.");
  await app.press("Cancel join request");
  expect(app.texts()).not.toContain("Waiting for a group admin to approve your request.");
});

test("desktop chat search filters conversations", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.type("Search", "Mira");
  expect(app.texts()).toContain("Mira");
  expect(app.texts()).not.toContain("Ada");
});

test("New chat opens a chooser and People rows open conversations", async () => {
  const app = await open(".", { platform: "linux", width: 1200, height: 800 });
  await app.press("New chat");
  expect(app.texts()).toContain("New conversation");
  await app.press("Back");
  await app.press("Contacts");
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
  expect(app.texts()).toContain("Contacts");
  expect(app.texts()).toContain("Settings");
  await app.press("New chat");
  expect(app.texts()).toContain("New conversation");
  expect(app.texts()).not.toContain("Contacts");
  expect(app.texts()).not.toContain("Settings");
  await app.back();
  expect(app.texts()).toContain("Chats");
  expect(app.texts()).toContain("Contacts");
  expect(app.texts()).toContain("Settings");
  await app.press("Ada");
  // The conversation screens shows the peer's messages.
  expect(app.texts()).toContain("I pushed the new build — try it now.");
  expect(app.texts()).not.toContain(
    "Prototype only · no network connection; sends stay on this device.",
  );
  await app.type("Message", "A message from mobile");
  await app.press("Send");
  expect(app.texts()).toContain("A message from mobile");
  await app.type("Message", "Another message from mobile");
  await app.press("Send");
  expect(app.texts()).toContain("Another message from mobile");
  expect(app.texts()).not.toContain("Contacts");
  expect(app.texts()).not.toContain("Settings");
  // The chat list is behind the pushed screen.
  await app.back();
  expect(app.texts()).toContain("Chats");
  expect(app.texts()).toContain("Contacts");
  expect(app.texts()).toContain("Settings");
});
