# Polygon Implementation Summary

### 1. Inner Stroke Algorithm (Polygon Buffering)

Unlike Circles or Rectangles, Polygons lack a single "radius" to shrink. We implemented a **Parallel Offset Algorithm**:

- **Normals:** Calculated inward-facing normals for every edge.
    
- **Shift:** Moved edges inward by `width / 2`.
    
- **Intersection:** Calculated new vertices where shifted edges intersect (Line-Line intersection).
    

### 2. Bugs Encountered & Fixed

**Bug: Geometric Inversion ("The Size 20 Bug")**

- **The Issue:** When the stroke width was significantly larger than the polygon (e.g., Size 1, Stroke 20), the inward-shifted lines crossed over each other completely. The mathematical intersection points ended up far _outside_ the original shape, creating a massive "inverted" polygon.
    
- **The Fix (Safety Measure):** Implemented a **Bounding Box Constraint**.
    
    - We calculate the bounds of the original polygon and the new inner polygon.
        
    - If the inner polygon is **larger** than the original, we know inversion occurred.
        
    - **Fallback:** We discard the broken inner path and render a solid silhouette using the stroke color.
        

### 3. Known Issues (Remaining)

**Bug: Miter/Stroke Overflow**

- **The Issue:** On extremely thick strokes relative to the shape size, or on polygons with very sharp (acute) angles, the stroke geometry may still visually overflow the intended bounds or create artifacts, even if the bounding box check passes.
    
- **Status:** Deferred. Requires complex boolean operations or a more advanced offsetting library (like `cavalier_contours` or robust miter clipping) to solve perfectly for all non-convex shapes.
