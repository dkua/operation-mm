const element_timeline = document.getElementById("timeline");
/** @type {Element[]} */
let visible_timeline_groups = [];
let focus_nav_link_timeout;

// Work around Safari not having scrollend event
if ("onscrollend" in document) {
  document.addEventListener("scrollend", _ => {
    const reduced_motion = window.matchMedia("(prefers-reduced-motion)").matches;
    if (!reduced_motion) {
      document.querySelector(".timeline-nav .group-in-view").scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  });
} else {
  document.addEventListener("scroll", _ => {
    const reduced_motion = window.matchMedia("(prefers-reduced-motion)").matches;
    if (!reduced_motion) {
      if (focus_nav_link_timeout != null) {
        clearTimeout(focus_nav_link_timeout);
        focus_nav_link_timeout = null;
      }
      focus_nav_link_timeout = setTimeout(() => {
        document.querySelector(".timeline-nav .group-in-view").scrollIntoView({ behavior: "smooth", block: "nearest" });
        focus_nav_link_timeout = null;
      }, 50)
    }
  });
}
document.addEventListener("scroll", _ => {
  update_highlighted_timeline_link();
});

// Set up intersection observer on timeline groups to track which ones are within the visible area
const observer_timeline_groups = new IntersectionObserver((entries, _) => {
  for (const e of entries) {
    if (e.isIntersecting) {
      visible_timeline_groups.push(e.target);
    } else {
      const index_remove_group = visible_timeline_groups.indexOf(e.target);
      if (index_remove_group >= 0) {
        visible_timeline_groups.splice(index_remove_group, 1);
      }
    }
  }
  if (document.querySelector(".timeline-nav .group-in-view") == null) {
    update_highlighted_timeline_link();
  }
}, { root: document.documentElement, threshold: 0 });
for (const el of document.getElementsByClassName("timeline-group")) {
  observer_timeline_groups.observe(el);
}

function update_highlighted_timeline_link() {
  const timeline_mid_y = document.documentElement.clientHeight / 2;
  for (const el of visible_timeline_groups) {
    const el_bounds = el.getBoundingClientRect();
    if (timeline_mid_y >= el_bounds.top && timeline_mid_y <= el_bounds.bottom) {
      // It's possible to land in a gap between groups, and we want to keep the last highlight in that case
      for (const el of document.querySelectorAll(".timeline-nav .group-in-view")) {
        el.classList.remove("group-in-view");
      }
      const new_highlight_el = document.getElementById(`timeline-link-${el.id}`);
      new_highlight_el.classList.add("group-in-view");
    }
  }
}
