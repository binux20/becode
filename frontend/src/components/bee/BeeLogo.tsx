import { motion } from 'framer-motion';

interface Props {
  size?: number;
  animated?: boolean;
}

export function BeeLogo({ size = 32, animated = false }: Props) {
  return (
    <motion.div
      animate={animated ? {
        y: [0, -3, 0],
        rotate: [0, -5, 5, 0],
      } : {}}
      transition={{
        duration: 1.5,
        repeat: Infinity,
        ease: 'easeInOut',
      }}
      className="relative"
      style={{ width: size, height: size }}
    >
      {/* Bee emoji with glow effect */}
      <span
        className="text-center block"
        style={{
          fontSize: size * 0.9,
          lineHeight: 1,
          filter: animated ? 'drop-shadow(0 0 8px rgba(255, 200, 0, 0.5))' : 'none',
        }}
      >
        🐝
      </span>

      {/* Animated wings (optional decorative effect) */}
      {animated && (
        <>
          <motion.div
            animate={{ opacity: [0.3, 0.6, 0.3], scale: [1, 1.2, 1] }}
            transition={{ duration: 0.3, repeat: Infinity }}
            className="absolute -top-1 -left-1 w-3 h-3 bg-bee-yellow/30 rounded-full blur-sm"
          />
          <motion.div
            animate={{ opacity: [0.3, 0.6, 0.3], scale: [1, 1.2, 1] }}
            transition={{ duration: 0.3, repeat: Infinity, delay: 0.15 }}
            className="absolute -top-1 -right-1 w-3 h-3 bg-bee-yellow/30 rounded-full blur-sm"
          />
        </>
      )}
    </motion.div>
  );
}

// Full animated bee mascot for welcome screen or loading states
export function BeeMascot({ size = 120 }: { size?: number }) {
  return (
    <motion.div
      initial={{ scale: 0, rotate: -180 }}
      animate={{ scale: 1, rotate: 0 }}
      transition={{ type: 'spring', duration: 0.8, bounce: 0.5 }}
      className="relative"
    >
      {/* Glow effect */}
      <motion.div
        animate={{
          scale: [1, 1.2, 1],
          opacity: [0.3, 0.5, 0.3],
        }}
        transition={{ duration: 2, repeat: Infinity }}
        className="absolute inset-0 bg-bee-yellow rounded-full blur-xl"
        style={{ width: size, height: size }}
      />

      {/* Main bee */}
      <motion.div
        animate={{
          y: [0, -10, 0],
        }}
        transition={{
          duration: 2,
          repeat: Infinity,
          ease: 'easeInOut',
        }}
        className="relative z-10"
      >
        <span style={{ fontSize: size }}>🐝</span>
      </motion.div>

      {/* Sparkles */}
      {[...Array(5)].map((_, i) => (
        <motion.div
          key={i}
          animate={{
            scale: [0, 1, 0],
            opacity: [0, 1, 0],
            x: [0, (Math.random() - 0.5) * 60],
            y: [0, (Math.random() - 0.5) * 60],
          }}
          transition={{
            duration: 1.5,
            repeat: Infinity,
            delay: i * 0.3,
          }}
          className="absolute top-1/2 left-1/2 w-2 h-2 bg-bee-yellow rounded-full"
          style={{
            transform: `translate(-50%, -50%)`,
          }}
        />
      ))}
    </motion.div>
  );
}
