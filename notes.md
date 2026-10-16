need a new name...

features:
 - nekos.moe fetch
 - fetch on a background thread
 - refetch image in app
 - select a folder
 - save the current image to the folder
 - the image viewer's state is now handled by the app, not reseting on rerender
 - logging to a file (info level) and to the console (warn level), and keeping two log files

 - keybind subscription function
 - timer for fetching

 - the code is unoptimized and error handling will be reworked


To fix:
 - first fetch takes longer
 - config should delete unused values on loading
 - move the full State out of the image viewer, so it won't cause bugs on rerender
 - app is for sure not responsive
 - do something with floats (before saving to file)   0.3999999999999999999 haha
 - could use anyhow ig (though I don't see it as important) (now I do)
 - show local images
 - disable the fetching button if a fetch is already in progress


Ideas:
 - see which website api is online
 - log level option in config


Sites:
 - Nekos.best
 - Danbooru
 - Waifu.im