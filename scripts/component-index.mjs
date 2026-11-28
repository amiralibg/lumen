// Print the README's component index from site/assets/content.js, so the
// README, the site and the count never disagree.
//   node scripts/component-index.mjs            # the markdown table
//   node scripts/component-index.mjs --count    # just the number
import { COMPONENTS, GROUPS } from "../site/assets/content.js";

const SITE = "https://amiralibg.github.io/lumen/#/components";
if (process.argv.includes("--count")) {
  console.log(COMPONENTS.length);
} else {
  console.log(`${COMPONENTS.length} components in ${GROUPS.length} groups. Each name links to its live page.\n`);
  console.log("| Group | Components |\n|---|---|");
  for (const g of GROUPS) {
    const list = COMPONENTS.filter((c) => c.group === g);
    console.log(`| ${g} (${list.length}) | ${list.map((c) => `[${c.name}](${SITE}/${c.slug})`).join(", ")} |`);
  }
}
