"use client";

import { useState } from "react";
import Link from "next/link";
import { Menu, X } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
  SheetClose,
} from "@/components/ui/sheet";
import { ThemeToggle } from "./ThemeToggle";

const navLinks = [
  { name: "Features", href: "/#features" },
  { name: "Architecture", href: "/#architecture" },
  { name: "Network", href: "/nodes" },
  { name: "Platforms", href: "/#platforms" },
  { name: "Docs", href: "/docs" },
];

export const Navbar = () => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <header className="fixed top-4 left-0 right-0 w-full z-50 flex justify-center px-4 sm:px-6 pointer-events-none">
      <nav className="pointer-events-auto w-full max-w-7xl mx-auto flex items-center justify-between h-16 px-6 bg-background/80 backdrop-blur-md border border-border/50 rounded-2xl shadow-sm">
        <Link href="/" className="flex items-center gap-2">
          <div className="w-6 h-6 rounded bg-foreground flex items-center justify-center font-bold text-background text-[10px]">A</div>
          <span className="text-base font-heading font-bold tracking-tight">Axon</span>
        </Link>
        
        {/* Desktop Links - Centered */}
        <div className="hidden md:flex absolute left-1/2 -translate-x-1/2 items-center gap-6 lg:gap-8 text-sm font-medium text-muted-foreground">
          {navLinks.map((link) => (
            <Link key={link.name} href={link.href} className="hover:text-foreground transition-colors">
              {link.name}
            </Link>
          ))}
        </div>

        <div className="flex items-center gap-2 sm:gap-4">
          <ThemeToggle />

          {/* Mobile Menu */}
          <div className="md:hidden">
            <Sheet open={isOpen} onOpenChange={setIsOpen}>
              <SheetTrigger asChild>
                <Button variant="ghost" size="icon" className="h-9 w-9 ml-1">
                  <Menu className="h-5 w-5" />
                  <span className="sr-only">Toggle menu</span>
                </Button>
              </SheetTrigger>
              <SheetContent side="right" className="w-[300px] sm:w-[400px] bg-background border-l border-border p-6 sm:p-8 pt-10 sm:pt-12" showCloseButton={false}>
                <div className="flex flex-row items-center justify-between border-b border-border/50 pb-6 mb-8">
                  <SheetTitle className="flex items-center gap-4 m-0 p-0">
                    <div className="w-8 h-8 rounded bg-foreground flex items-center justify-center font-bold text-background text-xs">A</div>
                    <span className="text-xl font-heading font-bold tracking-tight">Axon</span>
                  </SheetTitle>
                  <div className="flex items-center gap-3">
                    <ThemeToggle />
                    <SheetClose asChild>
                      <Button variant="ghost" size="icon" className="rounded-full h-9 w-9 border border-border/50">
                        <X className="h-4 w-4" />
                        <span className="sr-only">Close</span>
                      </Button>
                    </SheetClose>
                  </div>
                </div>
                <div className="flex flex-col gap-4">
                  {navLinks.map((link) => (
                    <Link
                      key={link.name}
                      href={link.href}
                      onClick={() => setIsOpen(false)}
                      className="text-lg font-medium text-muted-foreground hover:text-foreground transition-colors py-2 pl-12"
                    >
                      {link.name}
                    </Link>
                  ))}
                </div>
              </SheetContent>
            </Sheet>
          </div>
        </div>

      </nav>
    </header>
  );
};
