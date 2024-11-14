import {
  Controls,
  ReactFlow,
  ReactFlowProvider,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
  Handle,
  Position,
  MarkerType,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { useEffect } from "react";
import dagre from "@dagrejs/dagre";
import { Note } from "../note/Note";
import "@xyflow/react/dist/style.css";
import { useStore } from "../Store";
import { SharedNote } from "@bindings/SharedNote";
import { TempoResult } from "@bindings/TempoResult";

const nodeTypes = {
  note: NoteNode,
};

const NODE_WIDTH = 700;
const NODE_HEIGHT = 300;

interface NodeData {
  channelUlid: string | null;
  noteUlid: string;
  note: TempoResult<SharedNote>;
}

function NoteNode({ data }: { data: NodeData }) {
  return (
    <div style={{ width: `${NODE_WIDTH}px` }}>
      <Handle type="target" position={Position.Top} style={{ opacity: 0 }} />
      <Note
        channelUlid={data.channelUlid}
        noteUlid={data.noteUlid}
        note={data.note}
        noteBottom="jump"
      />
      <Handle type="source" position={Position.Bottom} style={{ opacity: 0 }} />
    </div>
  );
}

function Flow() {
  const [channelUlid, folderData] = useStore((state) => [
    state.channelUlid,
    state.folderData,
  ]);

  // const updateNodeInternals = useUpdateNodeInternals();
  // const nodesInitialized = useNodesInitialized();

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  useEffect(() => {
    if (!folderData) return;

    setNodes([]);
    setEdges([]);

    const notes = channelUlid
      ? folderData.channels[channelUlid]!.notes!
      : folderData.global!;

    let g = new dagre.graphlib.Graph();
    g.setGraph({ rankdir: "TB" });
    g.setDefaultEdgeLabel(() => ({}));

    let edges: Edge[] = [];
    let edgeCount = 0;

    Object.entries(notes).forEach(([ulid, doc]) => {
      if (!doc || "Err" in doc! || ("Ok" in doc! && !doc.Ok.attachment)) return;

      // not sure why typescript thinks doc could be undefined here
      // maybe im missing something?

      g.setNode(ulid, { width: NODE_WIDTH, height: NODE_HEIGHT });

      if (doc!.Ok.reply_ulid) {
        // edges.push([ulid, doc.Ok.reply_ulid]);
        edges.push({
          id: (edgeCount++).toString(),
          source: doc!.Ok.reply_ulid,
          target: ulid,
          markerEnd: {
            type: MarkerType.ArrowClosed,
          },
          style: { strokeWidth: 4 }
        });
        g.setEdge(doc!.Ok.reply_ulid, ulid);
      }
    });

    dagre.layout(g);

    const nodes: Node[] = [];

    g.nodes().forEach((noteUlid) => {
      const node = g.node(noteUlid);
      const x = node.x;
      const y = node.y;
      // const w = node.width;
      // const h = node.height;

      nodes.push({
        id: noteUlid,
        type: "note",
        data: {
          channelUlid,
          noteUlid,
          note: notes[noteUlid]!,
        },
        position: { x, y },
      });
    });

    setNodes(nodes);
    setEdges(edges);
  }, [channelUlid, folderData]);

  return (
    <ReactFlow
      fitView
      nodes={nodes}
      edges={edges}
      nodeTypes={nodeTypes}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      minZoom={0.1}
      nodesDraggable={false}
    >
      <Controls showInteractive={false} />
    </ReactFlow>
  );
}

export function TreeView() {
  return (
    <ReactFlowProvider>
      <Flow />
    </ReactFlowProvider>
  );
}
