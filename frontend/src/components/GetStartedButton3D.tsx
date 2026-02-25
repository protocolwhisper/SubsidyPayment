// @ts-nocheck
import { Canvas, useFrame } from "@react-three/fiber";
import { Suspense, useRef, useState } from "react";
import * as THREE from "three";

/**
 * ButtonShape Component
 * @param param0 
 * @returns 
 */
function ButtonShape({ isHovered }: { isHovered: boolean }) {
  const meshRef = useRef<THREE.Mesh>(null);
  useFrame((_, delta) => {
    if (!meshRef.current) return;
    meshRef.current.rotation.y += delta * 0.6;
    meshRef.current.rotation.x = THREE.MathUtils.lerp(
      meshRef.current.rotation.x,
      isHovered ? 0.2 : 0,
      0.08
    );
    const s = isHovered ? 1.15 : 1;
    meshRef.current.scale.lerp(new THREE.Vector3(s, s, s), 0.1);
  });

  return (
    <mesh ref={meshRef}>
      <cylinderGeometry args={[0.7, 0.7, 0.35, 24]} />
      <meshStandardMaterial
        color="#ffffff"
        emissive="#4A9EFF"
        emissiveIntensity={isHovered ? 0.32 : 0.16}
        metalness={0.4}
        roughness={0.35}
      />
    </mesh>
  );
}

/**
 * GetStartedButton3D Component
 * @param param0 
 * @returns 
 */
export function GetStartedButton3D({
  onClick,
  className = "",
}: {
  onClick: () => void;
  className?: string;
}) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      type="button"
      className={`get-started-3d-btn ${className}`}
      onClick={onClick}
      onPointerEnter={() => setHovered(true)}
      onPointerLeave={() => setHovered(false)}
      aria-label="Get Started"
    >
      <span className="get-started-3d-label">Get Started</span>
      <div className="get-started-3d-canvas">
        <Canvas
          camera={{ position: [0, 0, 2.5], fov: 40 }}
          dpr={[1, 2]}
          gl={{ alpha: true, antialias: true }}
        >
          <ambientLight intensity={0.6} />
          <directionalLight position={[2, 2, 3]} intensity={1.2} />
          <pointLight position={[-1, 1, 2]} intensity={0.5} color="#4A9EFF" />
          <Suspense fallback={null}>
            <ButtonShape isHovered={hovered} />
          </Suspense>
        </Canvas>
      </div>
    </button>
  );
}
