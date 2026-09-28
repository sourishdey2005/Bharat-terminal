import { Container } from "@/components/layout";
import { pageMeta } from "@/lib/seo";

export const metadata = pageMeta({ title: "License", description: "Bharat Terminal MIT license — copyright Sourish Dey. Free forever.", path: "/license" });

export default function LicensePage() {
  return (
    <Container className="max-w-3xl py-16">
      <h1 className="font-display text-4xl font-extrabold">MIT License</h1>
      <p className="mt-2 text-secondary">Copyright (c) 2026 Sourish Dey</p>
      <pre className="terminal-scroll mt-6 overflow-x-auto whitespace-pre-wrap rounded-xl border border-subtle bg-panel p-6 font-mono text-sm leading-6 text-secondary">{`Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.`}</pre>
    </Container>
  );
}
