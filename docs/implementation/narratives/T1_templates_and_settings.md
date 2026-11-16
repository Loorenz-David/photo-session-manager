# Phase T1_templates_and_settings: shooting templates can be made and kept, and Settings is now a list of sections

## What you can do now
Settings now has a list of sections down the side: Capture One, Camera, Export and previews, Shopify, Storage, Research (only when you use it) and the new **Shooting templates**. Every setting you already had is still there and behaves as before. Save settings still saves the first four sections.

In **Shooting templates** you make one template per item type, for example "Dining chair". You add example photos, and they are shown in black and white and numbered. You drag them into the order the listing should have. Per container you can switch Split on, pick a shape and set a zoom. You can also tap the containers in the order you shoot them. Every change is saved the moment you make it, and everything is still there after you quit and reopen the app. Nothing happens to your sessions or photos yet. Templates only start guiding a shoot in the next phases.

## Try it yourself
1. Start the app on scratch data:
   ```
   cd ~/Desktop/Developer/Photo-Session-Manager-agentic/apps/desktop
   PSM_DATA_DIR=$HOME/psm-templates-dev npm run tauri -- dev
   ```
2. Open Settings. A list of sections appears at the side. Click through them: every setting you know is there, and Save settings still saves the first four.
3. Click **Shooting templates**, type "Dining chair" and click **Create template**.
4. Click **Add photos** and choose four or five pictures of a chair. They appear in black and white, numbered.
5. Drag them into the order of the listing. On the third one, switch Split on and set Zoom to 20.
6. Click **Set shooting order** and tap the containers in the order you shoot them. Each one gets a second, small number. Click **Done**.
7. Quit the app and open it again (step 1). Everything is as you left it.
8. Click Add photos and choose a file that is not a picture. A sentence says so, and the template is unchanged.
9. Click **Delete template**. The app asks first. Click **Cancel**, and the template is still there. Delete only happens when you click Delete template inside that question.

## Choices made for you
- Templates are kept in the app's database on this Mac, not in the settings file. Each change saves by itself, and the other sections' Save button is not involved.
- A type matches regardless of capitals and spaces at the ends: "Dining chair" and "dining chair " are the same template. The name is shown as you first typed it.
- Example photos are copied into the app's data folder as JPEG, at most 1600 px on the long side. Your original file is never touched. JPEG, PNG, TIFF and WebP work. A file the app cannot read, such as a RAW or a HEIC, is refused with a sentence and nothing is lost.
- Examples are always shown in black and white. The stored copy keeps its colour, so colour could be shown later. Say so if you want colour anywhere.
- You can add a container without a photo. It then shows only its number.
- A new container is shot last until you set the shooting order again. Removing a container closes the gap in both orders.
- Tapping the shooting order saves after every tap. The containers you tapped come first, in tap order, and the rest follow in their earlier order.
- Zoom is a percentage from 0 to 80. 20 means the crop keeps 80 % of the width and height, centred.
- Shape "free" is not offered as a preset, because it has no crop of its own.
- The list of types offers the types you already made templates for, plus your shop's types when a shop is connected. Templates work without a shop.
- The Shooting templates section loads its data only when you open it.
- If deleting an old example file fails after a change was saved, the change stays saved and the screen shows what is stored. An unused copy may then be left in the data folder.
- **Changed during review:** the first version deleted a template the moment you pressed Delete template. Inside the app the "are you sure?" question never appeared. Delete template now opens the app's own question box with Cancel and Delete template, the same kind of box Settings already uses for unsaved changes. Nothing is deleted until you confirm.

## Not yet
- Templates do nothing during a session yet. Placing photos into containers, the faded placeholders in a session and the shooting order guidance arrive in the next template phases.
- Error messages for two rare cases ("a template already exists", "no template") still show a short code instead of a sentence.
- The Zoom box saves on every keystroke. If you type quickly, the number may jump back for a moment before it settles.

## Needs you
Nothing.
