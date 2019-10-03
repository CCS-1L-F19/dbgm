# Desktop Background Manager Design Document

- Based around 'background sets', which consist of a collection of 'background sources.' Think iTunes for desktop backgrounds.
    - A background set is associated with a particular screen resolution.
    - Background sets can be duplicated.
    - At first the only kind of backround source can be a folder on the filesystem with image files.
    - In the future, background sources could include image-hosting websites or RSS feeds or something.
    
- Left pane deals with a single background, displaying information about it and an overlay for choosing the area to display as an actual background.
    - All editing is nondestructive, source files will never be altered by the program. 
    - Files that have never been edited are marked as such and this attribute can be sorted by.
    - It should be possible to remove individual backgrounds from a source.
    - Can preview background on screen with button, restoring previous settings afterwards.

- Right pane lists all sources and backgrounds in the current set, used to add/remove sets and backgrounds and to select which background to edit.
    - List entry has thumbnail, filename, type (size and format), button to remove.
    - TODO: Decide behavior when new backgrounds are added to a source. The program could:
        - Immediately import new backgrounds, marked as "unedited"
        - Inform the user new backgrounds are available and ask if they should be imported.
        - Include an explicit "reimport source" feature, allowing the user to choose when new backgrounds are imported. 
    - Warn user when the contents of an existing background's source file has changed or been deleted and ask the user what to do.

- Utilities:
    - Integration with image resizing services like waifu2x?
    - Convert all background files to lossless format
    - Rebuild backgrounds in case cache gets messed up
    - Make all actions undoable/redoable
    - Edit currently displayed background from desktop context menu
    - Consolidate sources?
    