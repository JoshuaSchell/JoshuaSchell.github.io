export interface WorkExperience {
  title: string;
  company: string;
  location?: string;
  startDate: string;
  endDate?: string;
  description: string;
  url?: string;
}

export const workExperience: WorkExperience[] = [
  {
    title: "Software Engineer Intern",
    company: "Collins Aerospace",
    location: "Salt Lake City, UT",
    startDate: "May 2025",
    endDate: "present",
    description:
      "Building scalable, distributed systems for creating and managing dynamic Software-Defined Networks.",
    url: "https://www.collinsaerospace.com",
  },
  // TODO: Add more work experiences
] as const;
