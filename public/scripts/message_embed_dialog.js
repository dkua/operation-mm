/** @type {HTMLDialogElement} */
const dialog = document.getElementById("video-dialog");

/**
  * @param {string} video_id
  *
  */
function open_video_dialog(video_id) {
    const video_iframe = dialog.querySelector("iframe");
    video_iframe.src = `https://www.youtube-nocookie.com/embed/${video_id}?autoplay=1`;
    dialog.showModal();
}

dialog.addEventListener("pointerdown", (e) => {
    if (e.target === dialog) {
        window.addEventListener("pointerup", (e) => {
            if (e.target === dialog) {

                dialog.close();
            }
        }, { once: true });
    }
});
dialog.addEventListener("close", () => {
    const video_iframe = dialog.querySelector("iframe");
    video_iframe.src = "";
});
dialog.querySelector(".dialog-close-button").addEventListener("click", () => dialog.close());

for (const el of document.getElementsByClassName('video placeholder')) {
    el.addEventListener('click', () => open_video_dialog(el.getAttribute('data-video_id')));
}
