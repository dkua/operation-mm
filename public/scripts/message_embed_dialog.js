{
    /** @type {HTMLDialogElement} */
    const dialog = document.getElementById("video-dialog");
    const dialog_loader = dialog.querySelector(".loader");
    let timeout_show_loader;

    /**
     * @param {string} video_id
     */
    const open_video_dialog = (video_id, clip_id, clipt) => {
        let video_iframe = dialog.querySelector("iframe");
        if (clip_id != null && clipt != null) {
            video_iframe.src = `https://www.youtube.com/embed/${video_id}?autoplay=1&clip=${clip_id}&clipt=${clipt}`;
        } else {
            video_iframe.src = `https://www.youtube-nocookie.com/embed/${video_id}?autoplay=1`;
        }
        dialog.showModal();
        timeout_show_loader = setTimeout(() => dialog_loader.style.visibility = "visible", 500);
    }

    /**
     * @type EventListener
     */
    const handle_video_click = (ev) => {
        const video_id = ev.currentTarget.getAttribute("data-video_id");
        const clip_id = ev.currentTarget.getAttribute("data-clip_id");
        const clipt = ev.currentTarget.getAttribute("data-clipt");
        open_video_dialog(video_id, clip_id, clipt);
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
