import { Sidebar, Header, Breadcrumb, CommandPalette } from "@/components/layout";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-h-screen">
      <Sidebar />
      <main className="flex-1 flex flex-col">
        <Header />
        <div className="flex-1 p-6">
          <Breadcrumb />
          {children}
        </div>
      </main>
      <CommandPalette />
    </div>
  );
}
