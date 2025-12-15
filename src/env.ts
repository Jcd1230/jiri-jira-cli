declare const process: any;

function panic(msg: string) {
  console.error(msg);
  console.error("Required environment variables:");
  console.error("  JIRA_API_USERNAME - your Atlassian account email");
  console.error("  JIRA_API_TOKEN    - API token from https://id.atlassian.com/manage-profile/security/api-tokens");
  console.error("  JIRA_SITE         - Base Jira site URL, e.g. https://your-org.atlassian.net");
  process.exit(1);
}

const user = process.env.JIRA_API_USERNAME;
const token = process.env.JIRA_API_TOKEN;
const site = process.env.JIRA_SITE;

if (!user || !token || !site) {
  panic("Missing environment: JIRA_API_USERNAME, JIRA_API_TOKEN, and JIRA_SITE are all required.");
  throw new Error("unreachable");
}

export const env = { user, token, site };
