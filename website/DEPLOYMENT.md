# Deployment (free tier)

## Vercel (primary)
1. Push repo to GitHub.
2. vercel.com → New Project → select repo → set **Root Directory** to `website`.
3. Framework preset: Next.js. No env vars required.
4. Deploy. Every push auto-deploys.

Custom domain: Vercel → Settings → Domains → add CNAME via Cloudflare.

## Alternatives
- **Cloudflare Pages**: connect repo, build `npm run build`, output `.next` via `@cloudflare/next-on-pages` — or static export.
- **Netlify**: base `website/`, build `npm run build`, plugin `netlify-plugin-nextjs`.
- **GitHub Pages**: `next export` static mode only (live API routes disabled).

## Integrations (all free)
- Vercel Analytics + Speed Insights (included, no keys).
- Newsletter: Resend (`RESEND_API_KEY`, 3,000 emails/mo free).
- Contact: Formspree or Resend (50–3,000/mo free).

Made by Sourish Dey.
