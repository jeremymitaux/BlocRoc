import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'BlocRoc',
  description: 'Decentralized event ticketing on the blockchain',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
