export interface SocialLink {
  name: string;
  url: string;
  icon: string;
}

export const socialLinks: SocialLink[] = [
  {
    name: "Email",
    url: "mailto:contact@joshuaschell.com",
    icon: "mail",
  },
  {
    name: "x.com",
    url: "https://x.com/hoshuatxt",
    icon: "x",
  },
  {
    name: "GitHub",
    url: "https://github.com/JoshuaSchell",
    icon: "github",
  },
  {
    name: "LinkedIn",
    url: "https://www.linkedin.com/in/josh-schell/",
    icon: "linkedin",
  },
] as const;
