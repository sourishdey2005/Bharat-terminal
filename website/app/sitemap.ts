import type { MetadataRoute } from "next";
import { SITE } from "@/lib/utils";

export default function sitemap(): MetadataRoute.Sitemap {
  const paths = ["", "/features", "/download", "/docs", "/docs/install", "/docs/cli", "/docs/companies", "/docs/api", "/pricing", "/about", "/blog", "/blog/launch", "/changelog", "/roadmap", "/faq", "/contact", "/privacy", "/terms", "/license"];
  return paths.map((p) => ({ url: `${SITE.url}${p || "/"}`, lastModified: new Date(), changeFrequency: "weekly", priority: p === "" ? 1 : 0.7 }));
}
