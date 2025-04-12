## MaiMap后端服务

- 使用Salvo框架，MongoDB数据库

## API文档

[ApiFox文档](https://knqhhjuvxm.apifox.cn)

## 开发

## 部署运行

### 环境变量

```dotenv
#.env
QMAP_KEY=腾讯地图ApiKey
DATABASE_URI=mongodb://host.docker.internal
BACKUP_PATH=/app/
ALI_ACCESS_KEY_ID=阿里云AccessKeyID
ALI_ACCESS_KEY_SECRET=阿里云AccessKeySecret
ALI_OSS_REGION=cn-beijing
ALI_OSS_ENDPOINT=oss-cn-beijing.aliyuncs.com
ALI_OSS_BUCKET_NAME=Bucket名称
```

其中，阿里云变量用于将数据库的备份上传到OSS上。

### 构建

使用仓库中的Dockerfile构建。

```shell
docker build -t maimap-backend \
  --build-arg ENV_FILE_URL= \ 
  # Github .env文件路径 
  # 如https://raw.githubusercontent.com/120MF/maimap-env/main/.env 
  --build-arg GITHUB_TOKEN= \
  # Github token
  .

```

### 运行

```shell
docker run --name mb --add-host=host.docker.internal:host-gateway -p 5800:5800 -d maimap-backend
```

`--add-host`用于让host上的mongodb服务能够被容器内部访问。

