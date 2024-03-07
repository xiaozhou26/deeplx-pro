# 指定node为基础镜像，在官方版本中选择alpine版本更小
FROM node:14-alpine

# 在容器内创建app目录并设置工作目录
WORKDIR /app

# 复制package.json和package-lock.json
COPY package*.json ./

# 安装app依赖包
RUN npm install --production

# 复制本地代码到容器内
COPY . .

# 暴露容器端口
EXPOSE 9000

# 启动node应用
CMD [ "node", "index.js" ]
