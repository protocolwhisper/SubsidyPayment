// @ts-nocheck
import { useRef } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import * as THREE from "three";

function AmbientShape() {
  const meshRef = useRef<THREE.Mesh>(null);
  const groupRef = useRef<THREE.Group>(null);
  useFrame((state, delta) => {
    if (groupRef.current) {
      groupRef.current.rotation.y += delta * 0.12;
      groupRef.current.rotation.x = Math.sin(state.clock.elapsedTime * 0.15) * 0.08;
    }
    if (meshRef.current) {
      meshRef.current.rotation.z += delta * 0.05;
    }
  });

  return (
    <group ref={groupRef}>
      <mesh ref={meshRef} position={[0, 0, -3]}>
        <torusGeometry args={[2.2, 0.6, 32, 64]} />
        <meshStandardMaterial
          color="#0d1117"
          emissive="#4A9EFF"
          emissiveIntensity={0.12}
          transparent
          opacity={0.48}
          wireframe
        />
      </mesh>
      <mesh position={[1.5, 0.8, -4]}>
        <sphereGeometry args={[1.2, 32, 32]} />
        <meshStandardMaterial
          color="#0a0a0f"
          emissive="#79F8C6"
          emissiveIntensity={0.07}
          transparent
          opacity={0.32}
          wireframe
        />
      </mesh>
      <mesh position={[-1.2, -0.5, -3.5]}>
        <torusKnotGeometry args={[0.8, 0.25, 64, 8]} />
        <meshStandardMaterial
          color="#0d1117"
          emissive="#7ab8ff"
          emissiveIntensity={0.10}
          transparent
          opacity={0.38}
          wireframe
        />
      </mesh>
    </group>
  );
}

export function LandingBackground3D() {
  return (
    <div className="landing-background-3d" aria-hidden>
      <Canvas
        camera={{ position: [0, 0, 5], fov: 50 }}
        dpr={[1, 1.5]}
        gl={{ alpha: true, antialias: true, powerPreference: "low-power" }}
      >
        <color attach="background" args={["#0a0a0f"]} />
        <ambientLight intensity={0.4} />
        <directionalLight position={[5, 5, 5]} intensity={0.3} />
        <pointLight position={[-3, 2, 2]} intensity={0.2} color="#4A9EFF" />
        <pointLight position={[3, -1, 2]} intensity={0.15} color="#79F8C6" />
        <AmbientShape />
      </Canvas>
    </div>
  );
}
