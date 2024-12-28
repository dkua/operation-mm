{
    const type_filter = document.getElementById("timeline__typefilter");
    const timeline = document.getElementById("timeline");
    const num_visible_events = document.getElementById("num_visible");
    const timeline_events = Array.from(timeline.children);
    const hidden_types = new Map();

    type_filter.addEventListener("change", function(el) {
        if (el.target.checked) {
            hidden_types.delete(el.target.value);
            timeline_events.forEach((te) => {
                if (hidden_types.has(te.getAttribute("event-type"))) {
                    return;
                } else {
                    timeline.append(te);
                };
            });
        } else {
            // Hide this event type
            const events_of_type = timeline.querySelectorAll(`[event-type=${el.target.value}]`);
            hidden_types.set(el.target.value, events_of_type.length);
            events_of_type.forEach((e) => e.remove());
        };
        num_visible_events.innerText = timeline.children.length;
    });
}
