{
    const msg_container = document.getElementById("msg-container");
    /** @type {HTMLTemplateElement} */
    const messages = document.getElementById("messages-template");
    for (const font of document.fonts) {
        // Have to use `includes` because browsers are inconsistent about having quotes in the family
        // TODO: Possibly load other Noto font chunks when we have the finalised messages. Depending on page load performance.
        if ((!font.family.includes("JP") && !font.family.includes("KR") && font.family.includes("Noto Sans"))
            || font.family.includes("Edo SZ")) {
            font.load();
        }
    }
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
