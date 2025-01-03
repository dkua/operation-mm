{
    const msg_container = document.getElementById("msg-container");
    /** @type {HTMLTemplateElement} */
    const messages = document.getElementById("messages-template");
    // TODO: Possibly load other Noto font chunks when we have the finalised messages. Depending on page load performance.
    document.fonts.load("1em Noto Sans");
    document.fonts.load("1em Edo SZ");
    document.fonts.ready.then(() => {
        // Normally template content is cloned before placing in the DOM, but in this case
        // the contents are only needed once
        msg_container.replaceChildren(messages.content);
        twemoji.parse(msg_container);
        const message_masonry = new Masonry(msg_container, {
            itemSelector: ".msg",
            gutter: 20,
            fitWidth: true,
            layoutInstant: true,
        });
    });
}
