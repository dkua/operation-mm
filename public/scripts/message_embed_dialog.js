{
    /** @type {HTMLDialogElement} */
    const dialog = document.getElementById("video-dialog");
    const dialog_loader = dialog.querySelector(".loader");
    let timeout_show_loader;

    /**
     * @param {string} video_id
     */
    const open_video_dialog = (video_id) => {
        const video_iframe = dialog.querySelector("iframe");
        video_iframe.src = `https://www.youtube-nocookie.com/embed/${video_id}?autoplay=1`;
        dialog.showModal();
        timeout_show_loader = setTimeout(() => dialog_loader.style.visibility = "visible", 500);
    }

    /**
     * @type EventListener
     */
    const handle_video_click = (ev) => {
        open_video_dialog(ev.currentTarget.getAttribute("data-video_id"));
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
        clearTimeout(timeout_show_loader);
        dialog_loader.style.visibility = "hidden";
    });
    dialog.querySelector(".dialog-close-button").addEventListener("click", () => dialog.close());

    /** @type {HTMLTemplateElement} */
    const messages = document.getElementById("messages-template");
    for (const el of messages.content.querySelectorAll('.video.placeholder')) {
        el.addEventListener('click', handle_video_click);
    }
}
