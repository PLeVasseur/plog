// Shim for Comprehensive Rust's speaker-notes.js when built with stock
// mdBook: provides the icon <template> elements that Comprehensive Rust
// defines in its custom index.hbs via its `fa` handlebars helper.
//
// Icon SVGs: Font Awesome Free 6, https://fontawesome.com
// License: CC BY 4.0 (icons), https://fontawesome.com/license/free
(function () {
  function addIconTemplate(id, viewBox, path) {
    if (document.getElementById(id)) {
      return;
    }
    let template = document.createElement("template");
    template.id = id;
    template.innerHTML =
      '<svg xmlns="http://www.w3.org/2000/svg" viewBox="' +
      viewBox +
      '" width="1em" height="1em" fill="currentColor" aria-hidden="true"><path d="' +
      path +
      '"/></svg>';
    document.body.appendChild(template);
  }

  addIconTemplate(
    "fa-xmark",
    "0 0 384 512",
    "M342.6 150.6c12.5-12.5 12.5-32.8 0-45.3s-32.8-12.5-45.3 0L192 210.7 86.6 105.4c-12.5-12.5-32.8-12.5-45.3 0s-12.5 32.8 0 45.3L146.7 256 41.4 361.4c-12.5 12.5-12.5 32.8 0 45.3s32.8 12.5 45.3 0L192 301.3 297.4 406.6c12.5 12.5 32.8 12.5 45.3 0s12.5-32.8 0-45.3L237.3 256 342.6 150.6z"
  );
  addIconTemplate(
    "fa-arrow-up-right-from-square",
    "0 0 512 512",
    "M352 0c-12.9 0-24.6 7.8-29.6 19.8s-2.2 25.7 6.9 34.9L370.7 96 201.4 265.4c-12.5 12.5-12.5 32.8 0 45.3s32.8 12.5 45.3 0L416 141.3l41.4 41.4c9.2 9.2 22.9 11.9 34.9 6.9s19.8-16.6 19.8-29.6V32c0-17.7-14.3-32-32-32H352zM80 32C35.8 32 0 67.8 0 112V432c0 44.2 35.8 80 80 80H400c44.2 0 80-35.8 80-80V320c0-17.7-14.3-32-32-32s-32 14.3-32 32V432c0 8.8-7.2 16-16 16H80c-8.8 0-16-7.2-16-16V112c0-8.8 7.2-16 16-16H192c17.7 0 32-14.3 32-32s-14.3-32-32-32H80z"
  );
})();
