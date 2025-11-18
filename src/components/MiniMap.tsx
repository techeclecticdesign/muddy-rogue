import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface MiniMapNode {
  x: number;
  y: number;
  room_key: string;
  room_name: string;
  is_player: boolean;
  connections: string[];
}

export default function MiniMap({ enabled }: { enabled: boolean }) {
  const [nodes, setNodes] = useState<MiniMapNode[]>([]);

  useEffect(() => {
    if (!enabled) return;

    const fetchMiniMap = async () => {
      try {
        const data = await invoke<MiniMapNode[]>("get_minimap");
        setNodes(data);
      } catch (error) {
        console.error("Failed to fetch minimap:", error);
      }
    };

    fetchMiniMap();

    const unlisten = listen("minimap-update", fetchMiniMap);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [enabled]);

  if (!enabled || nodes.length === 0) return null;

  // Calculate bounds
  const minX = Math.min(...nodes.map((n) => n.x));
  const maxX = Math.max(...nodes.map((n) => n.x));
  const minY = Math.min(...nodes.map((n) => n.y));
  const maxY = Math.max(...nodes.map((n) => n.y));

  const gridWidth = maxX - minX + 1;
  const gridHeight = maxY - minY + 1;
  const cellSize = 40;

  // Create a map for quick node lookup
  const nodeMap = new Map(nodes.map((n) => [n.room_key, n]));

  return (
    <div
      style={{
        position: "fixed",
        top: "20px",
        right: "20px",
        padding: "10px",
        borderRadius: "8px",
        opacity: 0.6,
      }}
    >
      <svg
        width={gridWidth * cellSize}
        height={gridHeight * cellSize}
        style={{ display: "block" }}
      >
        {/* Draw connection lines first (so they appear behind nodes) */}
        {nodes.map((node) => {
          const x1 = (node.x - minX) * cellSize + cellSize / 2;
          const y1 = (maxY - node.y) * cellSize + cellSize / 2;

          return node.connections.map((connectedKey) => {
            const connectedNode = nodeMap.get(connectedKey);
            if (!connectedNode) return null;

            const x2 = (connectedNode.x - minX) * cellSize + cellSize / 2;
            const y2 = (maxY - connectedNode.y) * cellSize + cellSize / 2;

            // Only draw each line once (from lower key to higher key)
            if (node.room_key > connectedKey) return null;

            return (
              <line
                key={`${node.room_key}-${connectedKey}`}
                x1={x1}
                y1={y1}
                x2={x2}
                y2={y2}
                stroke="#fff"
                strokeWidth="2"
                opacity="0.5"
              />
            );
          });
        })}

        {/* Draw nodes on top of lines */}
        {nodes.map((node) => {
          const x = (node.x - minX) * cellSize + cellSize / 2;
          const y = (maxY - node.y) * cellSize + cellSize / 2;

          return (
            <g key={node.room_key}>
              <circle
                cx={x}
                cy={y}
                r={node.is_player ? 15 : 10}
                fill={node.is_player ? "#4CAF50" : "#2196F3"}
                stroke="#fff"
                strokeWidth="2"
              />
            </g>
          );
        })}
      </svg>
    </div>
  );
}
