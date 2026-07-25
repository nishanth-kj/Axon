"use client";

import { useEffect, useState } from "react";
import { ChevronUp, ChevronDown, Activity, AlertCircle, Clock, Hash, Globe, Link2, RefreshCw } from "lucide-react";
import { formatDistanceToNow, parseISO } from "date-fns";
import { Navbar } from "@/components/Navbar";
import { Footer } from "@/components/Footer";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Card } from "@/components/ui/card";

export default function NodesPage() {
  const [nodes, setNodes] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  
  const [page, setPage] = useState(1);
  const [size, setSize] = useState(10);
  const [sortBy, setSortBy] = useState("registered_at");
  const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc");
  const [pagination, setPagination] = useState({ total: 0, totalPages: 1 });

  const fetchNodes = async () => {
    setLoading(true);
    try {
      const response = await fetch("/api/v1/nodes/details", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ page, size, sortBy, sortOrder })
      });
      const result = await response.json();
      if (result.status === 1 && result.data) {
        setNodes(result.data.nodes);
        setPagination(result.data.pagination);
      }
    } catch (error) {
      console.error("Failed to fetch nodes:", error);
    }
    setLoading(false);
  };

  useEffect(() => {
    fetchNodes();
  }, [page, size, sortBy, sortOrder]);

  const handleSort = (column: string) => {
    if (sortBy === column) {
      setSortOrder(sortOrder === "asc" ? "desc" : "asc");
    } else {
      setSortBy(column);
      setSortOrder("desc");
    }
  };

  const renderSortIcon = (column: string) => {
    if (sortBy !== column) return null;
    return sortOrder === "asc" ? <ChevronUp className="w-4 h-4 inline ml-1" /> : <ChevronDown className="w-4 h-4 inline ml-1" />;
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case "ACTIVE":
        return <Badge variant="default" className="bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500/20 hover:text-emerald-500 border-emerald-500/20 border">ACTIVE</Badge>;
      case "PENDING":
        return <Badge variant="secondary" className="bg-amber-500/10 text-amber-500 hover:bg-amber-500/20 hover:text-amber-500 border-amber-500/20 border">PENDING</Badge>;
      default:
        return <Badge variant="destructive" className="bg-red-500/10 text-red-500 hover:bg-red-500/20 hover:text-red-500 border-red-500/20 border">{status}</Badge>;
    }
  };

  return (
    <div className="flex flex-col min-h-screen bg-background selection:bg-foreground selection:text-background">
      <Navbar />

      <main className="flex-1 w-full max-w-7xl mx-auto pt-36 pb-24 px-10 sm:px-12">
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 mb-8">
          <div>
            <h1 className="text-3xl font-bold font-heading flex items-center gap-3">
              <Globe className="w-8 h-8 text-primary" />
              Network Nodes
            </h1>
            <p className="text-muted-foreground mt-2">Active peers routing traffic on the Axon network</p>
          </div>
          <div className="flex flex-wrap items-center gap-3 sm:gap-4 w-full md:w-auto">
            <Badge variant="outline" className="px-4 py-2 text-sm rounded-xl whitespace-nowrap">
              <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse mr-2" />
              {pagination.total} Nodes Registered
            </Badge>
            <Button onClick={fetchNodes} disabled={loading} variant="outline" size="sm" className="gap-2 rounded-xl">
              <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
              Refresh
            </Button>
          </div>
        </div>

        <Card>
          <div className="overflow-x-auto">
            <Table className="min-w-[800px]">
              <TableHeader>
                <TableRow>
                  <TableHead className="cursor-pointer hover:text-foreground transition-colors whitespace-nowrap" onClick={() => handleSort("node_id")}>
                    <div className="flex items-center gap-2">
                      <Hash className="w-4 h-4" /> Node ID {renderSortIcon("node_id")}
                    </div>
                  </TableHead>
                  <TableHead className="cursor-pointer hover:text-foreground transition-colors whitespace-nowrap" onClick={() => handleSort("ip")}>
                    <div className="flex items-center gap-2">
                      <Link2 className="w-4 h-4" /> Endpoint {renderSortIcon("ip")}
                    </div>
                  </TableHead>
                  <TableHead className="cursor-pointer hover:text-foreground transition-colors whitespace-nowrap" onClick={() => handleSort("status")}>
                    <div className="flex items-center gap-2">
                      <Activity className="w-4 h-4" /> Status {renderSortIcon("status")}
                    </div>
                  </TableHead>
                  <TableHead className="cursor-pointer hover:text-foreground transition-colors whitespace-nowrap" onClick={() => handleSort("registered_at")}>
                    <div className="flex items-center gap-2">
                      <Clock className="w-4 h-4" /> Joined {renderSortIcon("registered_at")}
                    </div>
                  </TableHead>
                  <TableHead className="cursor-pointer hover:text-foreground transition-colors whitespace-nowrap" onClick={() => handleSort("last_seen")}>
                    <div className="flex items-center gap-2">
                      <AlertCircle className="w-4 h-4" /> Last Alive {renderSortIcon("last_seen")}
                    </div>
                  </TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {loading && nodes.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={5} className="h-32 text-center text-muted-foreground">
                      <div className="flex justify-center items-center gap-3">
                        <div className="w-5 h-5 border-2 border-primary border-t-transparent rounded-full animate-spin" />
                        Fetching nodes...
                      </div>
                    </TableCell>
                  </TableRow>
                ) : nodes.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={5} className="h-32 text-center text-muted-foreground">
                      No nodes found in the registry.
                    </TableCell>
                  </TableRow>
                ) : (
                  nodes.map((node) => (
                    <TableRow key={node.node_id} className="group">
                      <TableCell className="font-mono text-sm text-muted-foreground">
                        {node.node_id.substring(0, 16)}...
                      </TableCell>
                      <TableCell>
                        <div className="flex flex-col">
                          <span className="font-medium whitespace-nowrap">{node.ip}</span>
                          <span className="text-muted-foreground text-xs whitespace-nowrap">Port: {node.port}</span>
                        </div>
                      </TableCell>
                      <TableCell>
                        {getStatusBadge(node.status)}
                      </TableCell>
                      <TableCell className="text-sm text-muted-foreground whitespace-nowrap">
                        {new Date(node.registered_at).toLocaleString()}
                      </TableCell>
                      <TableCell className="text-sm whitespace-nowrap">
                        <Badge variant="outline" className="text-primary font-normal">
                          {formatDistanceToNow(parseISO(node.last_seen), { addSuffix: true })}
                        </Badge>
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </div>
          
          <div className="p-4 border-t flex flex-col md:flex-row items-center justify-between gap-6">
            <div className="text-sm text-muted-foreground text-center md:text-left w-full md:w-auto">
              Showing page <span className="font-medium text-foreground">{page}</span> of <span className="font-medium text-foreground">{Math.max(1, pagination.totalPages)}</span>
            </div>
            
            <div className="flex flex-wrap items-center justify-center md:justify-end gap-4 w-full md:w-auto">
              <div className="flex items-center gap-2">
                <Button 
                  variant="outline" 
                  size="sm"
                  onClick={() => setPage(p => Math.max(1, p - 1))}
                  disabled={page === 1}
                >
                  Previous
                </Button>
                
                <div className="flex gap-1 hidden sm:flex overflow-x-auto max-w-[200px]">
                  {[...Array(Math.max(1, pagination.totalPages))].map((_, i) => (
                    <Button
                      key={i}
                      variant={page === i + 1 ? "default" : "ghost"}
                      size="sm"
                      className="w-8 h-8 p-0 flex-shrink-0"
                      onClick={() => setPage(i + 1)}
                    >
                      {i + 1}
                    </Button>
                  ))}
                </div>

                <Button 
                  variant="outline"
                  size="sm"
                  onClick={() => setPage(p => Math.min(Math.max(1, pagination.totalPages), p + 1))}
                  disabled={page === Math.max(1, pagination.totalPages) || pagination.totalPages === 0}
                >
                  Next
                </Button>
              </div>
              
              <div className="hidden sm:block h-6 w-px bg-border"></div>

              <div className="flex items-center gap-2 text-sm">
                <span className="text-muted-foreground whitespace-nowrap">Rows:</span>
                <Select value={size.toString()} onValueChange={(v) => { setSize(Number(v)); setPage(1); }}>
                  <SelectTrigger className="w-[70px] h-9">
                    <SelectValue placeholder="10" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="5">5</SelectItem>
                    <SelectItem value="10">10</SelectItem>
                    <SelectItem value="20">20</SelectItem>
                    <SelectItem value="50">50</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          </div>
        </Card>
      </main>

      <Footer />
    </div>
  );
}
