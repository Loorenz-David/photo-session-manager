# Phase H2_list_pictures_pages_search: a picture on every row, pages of 25, and pictures in the search

## What you can do now
The sessions list no longer shows a broken "?" picture. A row shows this Mac's preview when its file is still here. For a product whose files the clean-up cleared ("Archived in Shopify"), the row shows a grey placeholder at first and then its first Shopify picture, a moment later and without reloading the list. That small picture is kept on this Mac, so it shows straight away the next time. The list now comes in pages of 25 with page numbers underneath. The search, the tabs, the time filter and "To review" still look through every session and take you back to page 1. In New session and Assign product, each search result shows its picture: this Mac's for your own products, the shop's for shop products. While a picture is loading, or when there is none, you see the first letter as before.

## Try it yourself
1. Open Terminal and run this step's tests, one after the other (each should end with `test result: ok`, or `passed` for the last one):

```
cd ~/Desktop/Developer/Photo-Session-Manager-agentic
cargo test -p psm-testkit --test suite h2_row_pictures::
cargo test -p psm-testkit --test suite h2_pages_and_search::
cargo test -p psm-desktop h2_
cd apps/desktop && npx vitest run src/features/sessions/__tests__/h2_list_pages_and_pictures.test.tsx src/features/sessions/__tests__/h2_search_pictures.test.tsx
```

2. In the installed app (once you install this build), open Sessions. Rows marked "Archived in Shopify" show a grey placeholder and, a moment later, their first picture. No row shows a broken "?".
3. With more than 25 sessions, page numbers appear under the list. Go to page 2 and type part of a title that is on page 1: it is found, and the list goes back to page 1.
4. Hover a row: the large card shows the same picture as the row.
5. Press N and type part of a product's name. Every result shows a picture (this Mac's or the shop's), or its first letter while the picture loads. Assign product works the same way.
6. Quit and reopen the app: the pictures are there at once.

## Choices made for you
- **A row never shows a broken picture.** It shows this Mac's preview, or the Shopify picture once it has been downloaded, or a neutral grey placeholder. Say "show the first letter instead of a placeholder" if you prefer that.
- **Pictures are downloaded only for the page you're looking at,** four at a time, after the list is already on screen. Say "fetch all pages' pictures" if you want them all ready in advance.
- **If Shopify no longer has a picture, the row stays a placeholder,** and Health says so once ("1 picture is no longer in the shop: <SKU>").
- **The search, the tabs, the time filter and "To review" look through every session.** Only the result is cut into pages.
- **Page numbers:** all of them up to 7 pages. With more, you see the first, the last and the pages either side of the one you're on, plus Previous and Next.
- **A shop product you haven't opened yet shows the placeholder in the list.** Its pictures are read the first time you open it. In the search it already shows the shop's picture.
- **Shop pictures for the search are kept small (160 px), in a store of at most 200 MB on this Mac.** The oldest go first. Say "make the search pictures bigger" or "keep more" to change it.
- **The two searches (your products and the shop) don't cancel each other's picture downloads.** Moving to another page does cancel the old page's downloads that haven't started yet.
- **The "To review · N" count covers every session,** not only the tab you're on. So on the Published tab it can show more than the rows you then see. Say "count only this tab" to change it.
- **Deleting a session also deletes that product's kept pictures on this Mac.** Nothing is asked of or changed in Shopify.
- **If you ask for a page that no longer exists** (say, after sessions were deleted), you get the last page and the page numbers follow.

## Not yet
- The "Refresh from Shopify" button, and the screen for a cleared or shop-only product, come in a later step. Until then, the Health line about a picture Shopify no longer has goes away only when you delete that session. It also disappears after the app restarts, even though the row stays a placeholder.
- Bringing a cleared product's photos back to this Mac to edit them is a later step.
- Only the Sessions page has pages. The pickers and the workspace's Sessions panel still show the whole list (the panel already uses the new placeholder).

## Needs you
Nothing.
